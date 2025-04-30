// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// use crate::util::{color::Color, sys};
// use crate::{
//     constants::{
//         BROTLI_COMPRESSION_LEVEL, EOF_PREFIX_NO_DICT, PROJECT_HASH_SECTION_NAME, RUST_TARGET,
//         TOOLCHAIN_FILE_NAME,
//     },
//     macros::*,
// };
// use brotli2::read::BrotliEncoder;
// use eyre::{bail, eyre, Result, WrapErr};
// use glob::glob;
// use std::{
//     env::current_dir,
//     fs,
//     io::Read,
//     path::{Path, PathBuf},
//     process,
//     sync::mpsc,
//     thread,
// };
// use std::{ops::Range, process::Command};
// use tiny_keccak::{Hasher, Keccak};
// use wasm_encoder::{Module, RawSection};
// use wasmparser::{Parser, Payload};

use eyre::{bail, eyre, Result, WrapErr};
use std::{
    env, fmt,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};
use toml::Value;

pub const RUST_TARGET: &str = "wasm32-unknown-unknown";

/// Define the optimization level when compiling the Stylus project.
/// The value `Unset` uses whichever config defined in Cargo.toml.
#[derive(Default, Clone)]
pub enum OptLevel {
    #[default]
    Unset,
    O0,
    O1,
    O2,
    O3,
    S,
    Z,
}

impl fmt::Display for OptLevel {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptLevel::Unset => write!(fmt, ""),
            OptLevel::O0 => write!(fmt, "0"),
            OptLevel::O1 => write!(fmt, "1"),
            OptLevel::O2 => write!(fmt, "2"),
            OptLevel::O3 => write!(fmt, "3"),
            OptLevel::S => write!(fmt, "s"),
            OptLevel::Z => write!(fmt, "z"),
        }
    }
}

#[derive(Default, Clone)]
pub struct BuildConfig {
    pub opt_level: OptLevel,
    pub stable: bool,
    pub features: Option<String>,
}

impl BuildConfig {
    pub fn new(stable: bool) -> Self {
        Self {
            stable,
            ..Default::default()
        }
    }
}

pub struct BuildResult {
    pub cargo_toml_version: String,
    pub wasm_file_path: PathBuf,
}

/// Build a Stylus project in the current directory to WASM.
pub fn build_wasm(cfg: BuildConfig) -> Result<BuildResult> {
    let cwd = env::current_dir().wrap_err("could not get current dir")?;

    // Enforce a version is included in the Cargo.toml file.
    let cargo_toml_path = cwd.join(Path::new("Cargo.toml"));
    let cargo_toml_version = extract_cargo_toml_version(&cargo_toml_path)?;
    let project_name = extract_cargo_project_name(&cargo_toml_path)?
        .replace("-", "_")
        .replace("\"", "");

    // Compile the contract with cargo.
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.arg("--lib");
    cmd.arg("--locked");
    if let Some(features) = cfg.features {
        cmd.arg(format!("--features={}", features));
    }
    if !cfg.stable {
        cmd.arg("-Z");
        cmd.arg("build-std=std,panic_abort");
        cmd.arg("-Z");
        cmd.arg("build-std-features=panic_immediate_abort");
    }
    if !matches!(cfg.opt_level, OptLevel::Unset) {
        cmd.arg("--config");
        cmd.arg(format!(
            "profile.release.opt-level='{}'",
            cfg.opt_level.to_string()
        ));
    }
    cmd.arg("--release");
    cmd.arg(format!("--target={RUST_TARGET}"));
    let output = cmd.output().wrap_err("failed to execute cargo build")?;
    if !output.status.success() {
        bail!(
            "cargo build failed: {}",
            String::from_utf8(output.stderr).unwrap_or("failed to decode output".to_owned())
        );
    }

    // Get the wasm file in the release directory.
    let release_path = cwd
        .join("target")
        .join(RUST_TARGET)
        .join("release")
        .join("deps");
    let release_files: Vec<PathBuf> = fs::read_dir(&release_path)
        .wrap_err("could not read release deps dir: {e}")?
        .filter_map(|r| r.ok())
        .map(|r| r.path())
        .filter(|r| r.is_file())
        .collect();
    let wasm_file_name = project_name + ".wasm";
    let wasm_file_path = release_files
        .into_iter()
        .find(|p| {
            if let Some(filename) = p.file_name() {
                filename.to_string_lossy().ends_with(&wasm_file_name)
            } else {
                false
            }
        })
        .ok_or(eyre!(
            "could not find WASM in release dir ({:?})",
            release_path
        ))?;

    let result = BuildResult {
        wasm_file_path,
        cargo_toml_version,
    };
    Ok(result)
}

/// Read the toolchain version from the rust-toolchain.toml file.
pub fn extract_toolchain_channel(toolchain_file_path: &PathBuf) -> Result<String> {
    let toolchain_file_contents = fs::read_to_string(toolchain_file_path).wrap_err(
        "expected to find a rust-toolchain.toml file in project directory \
        to specify your Rust toolchain for reproducible verification. The channel in your project's rust-toolchain.toml's \
        toolchain section must be a specific version e.g., '1.80.0' or 'nightly-YYYY-MM-DD'. \
        To ensure reproducibility, it cannot be a generic channel like 'stable', 'nightly', or 'beta'. Read more about \
        the toolchain file in https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file or see \
        the file in https://github.com/OffchainLabs/stylus-hello-world for an example",
    )?;
    let toolchain_toml: Value =
        toml::from_str(&toolchain_file_contents).wrap_err("failed to parse rust-toolchain.toml")?;

    // Extract the channel from the toolchain section
    let Some(toolchain) = toolchain_toml.get("toolchain") else {
        bail!("toolchain section not found in rust-toolchain.toml");
    };
    let Some(channel) = toolchain.get("channel") else {
        bail!("could not find channel in rust-toolchain.toml's toolchain section");
    };
    let Some(channel) = channel.as_str() else {
        bail!("channel in rust-toolchain.toml's toolchain section is not a string");
    };

    // Reject "stable" and "nightly" channels specified alone
    if channel == "stable" || channel == "nightly" || channel == "beta" {
        bail!("the channel in your project's rust-toolchain.toml's toolchain section must be a specific version e.g., '1.80.0' or 'nightly-YYYY-MM-DD'. \
        To ensure reproducibility, it cannot be a generic channel like 'stable', 'nightly', or 'beta'");
    }

    // Parse the Rust version from the toolchain project, only allowing alphanumeric chars and dashes.
    let channel = channel
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '.')
        .collect();

    Ok(channel)
}

fn extract_cargo_toml_version(cargo_toml_path: &Path) -> Result<String> {
    let cargo_toml_contents = fs::read_to_string(cargo_toml_path)
        .wrap_err("expected to find a Cargo.toml file in project directory")?;
    let cargo_toml: Value =
        toml::from_str(&cargo_toml_contents).wrap_err("failed to parse Cargo.toml")?;
    let Some(pkg) = cargo_toml.get("package") else {
        bail!("package section not found in Cargo.toml");
    };
    let Some(version) = pkg.get("version") else {
        bail!("could not find version in project's Cargo.toml [package] section");
    };
    let Some(version) = version.as_str() else {
        bail!("version in Cargo.toml's [package] section is not a string");
    };
    Ok(version.to_string())
}

fn extract_cargo_project_name(cargo_toml_path: &Path) -> Result<String> {
    let cargo_toml_contents = fs::read_to_string(cargo_toml_path)
        .wrap_err("expected to find a Cargo.toml file in project directory")?;
    let cargo_toml: Value =
        toml::from_str(&cargo_toml_contents).wrap_err("failed to parse Cargo.toml")?;
    let Some(pkg) = cargo_toml.get("package") else {
        bail!("package section not found in Cargo.toml");
    };
    let Some(name) = pkg.get("name") else {
        bail!("could not find name in project's Cargo.toml [package] section");
    };
    Ok(name.to_string())
}

fn read_file_preimage(filename: &Path) -> Result<Vec<u8>> {
    let mut contents = Vec::with_capacity(1024);
    {
        let filename = filename.as_os_str();
        contents.extend_from_slice(&(filename.len() as u64).to_be_bytes());
        contents.extend_from_slice(filename.as_encoded_bytes());
    }
    let mut file = File::open(filename)
        .wrap_err_with(|| format!("failed to open file: {}", filename.display()))?;
    contents.extend_from_slice(&file.metadata().unwrap().len().to_be_bytes());
    file.read_to_end(&mut contents)
        .wrap_err_with(|| format!("failed to read file {}", filename.display()))?;
    Ok(contents)
}

//fn hash_project(source_file_patterns: Vec<String>, cfg: BuildConfig) -> Result<[u8; 32]> {
//    let mut cmd = Command::new("cargo");
//    cmd.arg("--version");
//    let output = cmd
//        .output()
//        .map_err(|e| eyre!("failed to execute cargo command: {e}"))?;
//    if !output.status.success() {
//        bail!("cargo version command failed");
//    }
//    hash_files(&output.stdout, source_file_patterns, cfg)
//}

//pub fn hash_files(
//    cargo_version_output: &[u8],
//    source_file_patterns: Vec<String>,
//    cfg: BuildConfig,
//) -> Result<[u8; 32]> {
//    let mut keccak = Keccak::v256();
//    keccak.update(cargo_version_output);
//    if cfg.opt_level == OptLevel::Z {
//        keccak.update(&[0]);
//    } else {
//        keccak.update(&[1]);
//    }
//
//    // Fetch the Rust toolchain toml file from the project root. Assert that it exists and add it to the
//    // files in the directory to hash.
//    let toolchain_file_path = PathBuf::from(".").as_path().join(TOOLCHAIN_FILE_NAME);
//    let _ = std::fs::metadata(&toolchain_file_path).wrap_err(
//        "expected to find a rust-toolchain.toml file in project directory \
//         to specify your Rust toolchain for reproducible verification",
//    )?;
//
//    let mut paths = all_paths(PathBuf::from(".").as_path(), source_file_patterns)?;
//    paths.push(toolchain_file_path);
//    paths.sort();
//
//    // Read the file contents in another thread and process the keccak in the main thread.
//    let (tx, rx) = mpsc::channel();
//    thread::spawn(move || {
//        for filename in paths.iter() {
//            greyln!(
//                "File used for deployment hash: {}",
//                filename.as_os_str().to_string_lossy()
//            );
//            tx.send(read_file_preimage(filename))
//                .expect("failed to send preimage (impossible)");
//        }
//    });
//    for result in rx {
//        keccak.update(result?.as_slice());
//    }
//
//    let mut hash = [0u8; 32];
//    keccak.finalize(&mut hash);
//    greyln!(
//        "project metadata hash computed on deployment: {:?}",
//        hex::encode(hash)
//    );
//    Ok(hash)
//}

//fn all_paths(root_dir: &Path, source_file_patterns: Vec<String>) -> Result<Vec<PathBuf>> {
//    let mut files = Vec::<PathBuf>::new();
//    let mut directories = Vec::<PathBuf>::new();
//    directories.push(root_dir.to_path_buf()); // Using `from` directly

//    let glob_paths = expand_glob_patterns(source_file_patterns)?;

//    while let Some(dir) = directories.pop() {
//        for entry in fs::read_dir(&dir)
//            .map_err(|e| eyre!("Unable to read directory {}: {e}", dir.display()))?
//        {
//            let entry = entry.map_err(|e| eyre!("Error finding file in {}: {e}", dir.display()))?;
//            let path = entry.path();

//            if path.is_dir() {
//                if path.ends_with("target") || path.ends_with(".git") {
//                    continue; // Skip "target" and ".git" directories
//                }
//                directories.push(path);
//            } else if path.file_name().is_some_and(|f| {
//                // If the user has has specified a list of source file patterns, check if the file
//                // matches the pattern.
//                if !glob_paths.is_empty() {
//                    for glob_path in glob_paths.iter() {
//                        if glob_path == &path {
//                            return true;
//                        }
//                    }
//                    false
//                } else {
//                    // Otherwise, by default include all rust files, Cargo.toml and Cargo.lock files.
//                    f == "Cargo.toml" || f == "Cargo.lock" || f.to_string_lossy().ends_with(".rs")
//                }
//            }) {
//                files.push(path);
//            }
//        }
//    }
//    Ok(files)
//}

//fn expand_glob_patterns(patterns: Vec<String>) -> Result<Vec<PathBuf>> {
//    let mut files_to_include = Vec::new();
//    for pattern in patterns {
//        let paths = glob(&pattern)
//            .map_err(|e| eyre!("Failed to read glob pattern '{}': {}", pattern, e))?;
//        for path_result in paths {
//            let path = path_result.map_err(|e| eyre!("Error processing path: {}", e))?;
//            files_to_include.push(path);
//        }
//    }
//    Ok(files_to_include)
//}

//pub fn build_wasm_from_features(
//    features: Option<String>,
//    source_files: Vec<String>,
//) -> Result<(PathBuf, [u8; 32])> {
//    let toolchain_file_path = PathBuf::from(".").join(TOOLCHAIN_FILE_NAME);
//    let toolchain_channel = extract_toolchain_channel(&toolchain_file_path)?;
//    let rust_stable = !toolchain_channel.contains("nightly");
//    let mut cfg = BuildConfig::new(rust_stable);
//    cfg.features = features;
//    let wasm = build_dylib(cfg.clone())?;
//    let project_hash = hash_project(source_files, cfg)?;
//    Ok((wasm, project_hash))
//}

///// Reads a WASM file at a specified path and returns its brotli compressed bytes.
//pub fn compress_wasm(wasm: &PathBuf, project_hash: [u8; 32]) -> Result<(Vec<u8>, Vec<u8>)> {
//    let wasm =
//        fs::read(wasm).wrap_err_with(|| eyre!("failed to read Wasm {}", wasm.to_string_lossy()))?;
//
//    // We convert the WASM from binary to text and back to binary as this trick removes any dangling
//    // mentions of reference types in the wasm body, which are not yet supported by Arbitrum chain backends.
//    let wat_str =
//        wasmprinter::print_bytes(&wasm).map_err(|e| eyre!("failed to convert Wasm to Wat: {e}"))?;
//    let wasm = wasmer::wat2wasm(wat_str.as_bytes())
//        .map_err(|e| eyre!("failed to convert Wat to Wasm: {e}"))?;
//
//    // We include the project's hash as a custom section
//    // in the user's WASM so it can be verified by Cargo stylus'
//    // reproducible verification. This hash is added as a section that is
//    // ignored by WASM runtimes, so it will only exist in the file
//    // for metadata purposes.
//    let wasm = add_project_hash_to_wasm_file(&wasm, project_hash)
//        .wrap_err("failed to add project hash to wasm file as custom section")?;
//
//    let wasm =
//        strip_user_metadata(&wasm).wrap_err("failed to strip user metadata from wasm file")?;
//
//    let wasm = wasmer::wat2wasm(&wasm).wrap_err("failed to parse Wasm")?;
//
//    let mut compressor = BrotliEncoder::new(&*wasm, BROTLI_COMPRESSION_LEVEL);
//    let mut compressed_bytes = vec![];
//    compressor
//        .read_to_end(&mut compressed_bytes)
//        .wrap_err("failed to compress WASM bytes")?;
//
//    let mut contract_code = hex::decode(EOF_PREFIX_NO_DICT).unwrap();
//    contract_code.extend(compressed_bytes);
//
//    Ok((wasm.to_vec(), contract_code))
//}

//// Adds the hash of the project's source files to the wasm as a custom section
//// if it does not already exist. This allows for reproducible builds by cargo stylus
//// for all Rust stylus contracts. See `cargo stylus verify --help` for more information.
//fn add_project_hash_to_wasm_file(
//    wasm_file_bytes: &[u8],
//    project_hash: [u8; 32],
//) -> Result<Vec<u8>> {
//    let section_exists = has_project_hash_section(wasm_file_bytes)?;
//    if section_exists {
//        greyln!("Wasm file bytes already contains a custom section with a project hash, not overwriting'");
//        return Ok(wasm_file_bytes.to_vec());
//    }
//    Ok(add_custom_section(wasm_file_bytes, project_hash))
//}

//pub fn has_project_hash_section(wasm_file_bytes: &[u8]) -> Result<bool> {
//    let parser = wasmparser::Parser::new(0);
//    for payload in parser.parse_all(wasm_file_bytes) {
//        if let wasmparser::Payload::CustomSection(reader) = payload? {
//            if reader.name() == PROJECT_HASH_SECTION_NAME {
//                println!(
//                    "Found the project hash custom section name {}",
//                    hex::encode(reader.data())
//                );
//                return Ok(true);
//            }
//        }
//    }
//    Ok(false)
//}

//fn add_custom_section(wasm_file_bytes: &[u8], project_hash: [u8; 32]) -> Vec<u8> {
//    let mut bytes = vec![];
//    bytes.extend_from_slice(wasm_file_bytes);
//    wasm_gen::write_custom_section(&mut bytes, PROJECT_HASH_SECTION_NAME, &project_hash);
//    bytes
//}

//fn strip_user_metadata(wasm_file_bytes: &[u8]) -> Result<Vec<u8>> {
//    let mut module = Module::new();
//    // Parse the input WASM and iterate over the sections
//    let parser = Parser::new(0);
//    for payload in parser.parse_all(wasm_file_bytes) {
//        match payload? {
//            Payload::CustomSection { .. } => {
//                // Skip custom sections to remove sensitive metadata
//                greyln!("stripped custom section from user wasm to remove any sensitive data");
//            }
//            Payload::UnknownSection { .. } => {
//                // Skip unknown sections that might not be sensitive
//                println!("stripped unknown section from user wasm to remove any sensitive data");
//            }
//            item => {
//                // Handle other sections as normal.
//                if let Some(section) = item.as_section() {
//                    let (id, range): (u8, Range<usize>) = section;
//                    let data_slice = &wasm_file_bytes[range.start..range.end]; // Start at the beginning of the range
//                    let raw_section = RawSection {
//                        id,
//                        data: data_slice,
//                    };
//                    module.section(&raw_section);
//                }
//            }
//        }
//    }
//    // Return the stripped WASM binary
//    Ok(module.finish())
//}

//#[cfg(test)]
//mod test {
//    use super::*;
//    use std::{
//        env,
//        fs::{self, File},
//        io::Write,
//        path::Path,
//    };
//    use tempfile::{tempdir, TempDir};

//    #[cfg(feature = "nightly")]
//    extern crate test;

//    fn write_valid_toolchain_file(toolchain_file_path: &Path) -> Result<()> {
//        let toolchain_contents = r#"
//            [toolchain]
//            channel = "nightly-2020-07-10"
//            components = [ "rustfmt", "rustc-dev" ]
//            targets = [ "wasm32-unknown-unknown", "thumbv2-none-eabi" ]
//            profile = "minimal"
//        "#;
//        fs::write(&toolchain_file_path, toolchain_contents)?;
//        Ok(())
//    }

//    fn write_hash_files(num_files: usize, num_lines: usize) -> Result<TempDir> {
//        let dir = tempdir()?;
//        env::set_current_dir(dir.path())?;

//        let toolchain_file_path = dir.path().join(TOOLCHAIN_FILE_NAME);
//        write_valid_toolchain_file(&toolchain_file_path)?;

//        fs::create_dir(dir.path().join("src"))?;
//        let mut contents = String::new();
//        for _ in 0..num_lines {
//            contents.push_str("// foo");
//        }
//        for i in 0..num_files {
//            let file_path = dir.path().join(format!("src/f{i}.rs"));
//            fs::write(&file_path, &contents)?;
//        }
//        fs::write(dir.path().join("Cargo.toml"), "")?;
//        fs::write(dir.path().join("Cargo.lock"), "")?;

//        Ok(dir)
//    }

//    #[test]
//    fn test_extract_toolchain_channel() -> Result<()> {
//        let dir = tempdir()?;
//        let dir_path = dir.path();

//        let toolchain_file_path = dir_path.join(TOOLCHAIN_FILE_NAME);
//        let toolchain_contents = r#"
//            [toolchain]
//        "#;
//        std::fs::write(&toolchain_file_path, toolchain_contents)?;

//        let channel = extract_toolchain_channel(&toolchain_file_path);
//        let Err(err_details) = channel else {
//            panic!("expected an error");
//        };
//        assert!(err_details.to_string().contains("could not find channel"),);

//        let toolchain_contents = r#"
//            [toolchain]
//            channel = 32390293
//        "#;
//        std::fs::write(&toolchain_file_path, toolchain_contents)?;

//        let channel = extract_toolchain_channel(&toolchain_file_path);
//        let Err(err_details) = channel else {
//            panic!("expected an error");
//        };
//        assert!(err_details.to_string().contains("is not a string"),);

//        write_valid_toolchain_file(&toolchain_file_path)?;
//        let channel = extract_toolchain_channel(&toolchain_file_path)?;
//        assert_eq!(channel, "nightly-2020-07-10");
//        Ok(())
//    }

//    #[test]
//    fn test_all_paths() -> Result<()> {
//        let dir = tempdir()?;
//        let dir_path = dir.path();

//        let files = ["file.rs", "ignore.me", "Cargo.toml", "Cargo.lock"];
//        for file in files {
//            let file_path = dir_path.join(file);
//            let mut file = File::create(&file_path)?;
//            writeln!(file, "Test content")?;
//        }

//        let dirs = ["nested", ".git", "target"];
//        for d in dirs {
//            let subdir_path = dir_path.join(d);
//            if !subdir_path.exists() {
//                fs::create_dir(&subdir_path)?;
//            }
//        }

//        let nested_dir = dir_path.join("nested");
//        let nested_file = nested_dir.join("nested.rs");
//        if !nested_file.exists() {
//            File::create(&nested_file)?;
//        }

//        let found_files = all_paths(
//            dir_path,
//            vec![format!(
//                "{}/{}",
//                dir_path.as_os_str().to_string_lossy(),
//                "**/*.rs"
//            )],
//        )?;

//        // Check that the correct files are included
//        assert!(found_files.contains(&dir_path.join("file.rs")));
//        assert!(found_files.contains(&nested_dir.join("nested.rs")));
//        assert!(!found_files.contains(&dir_path.join("ignore.me")));
//        assert!(!found_files.contains(&dir_path.join("Cargo.toml"))); // Not matching *.rs
//        assert_eq!(found_files.len(), 2, "Should only find 2 Rust files.");

//        Ok(())
//    }

//    #[test]
//    pub fn test_hash_files() -> Result<()> {
//        let _dir = write_hash_files(10, 100)?;
//        let rust_version = "cargo 1.80.0 (376290515 2024-07-16)\n".as_bytes();
//        let hash = hash_files(rust_version, vec![], BuildConfig::new(false))?;
//        assert_eq!(
//            hex::encode(hash),
//            "06b50fcc53e0804f043eac3257c825226e59123018b73895cb946676148cb262"
//        );
//        Ok(())
//    }

//    #[cfg(feature = "nightly")]
//    #[bench]
//    pub fn bench_hash_files(b: &mut test::Bencher) -> Result<()> {
//        let _dir = write_hash_files(1000, 10000)?;
//        let rust_version = "cargo 1.80.0 (376290515 2024-07-16)\n".as_bytes();
//        b.iter(|| {
//            hash_files(rust_version, vec![], BuildConfig::new(false))
//                .expect("failed to hash files");
//        });
//        Ok(())
//    }
//}
////
