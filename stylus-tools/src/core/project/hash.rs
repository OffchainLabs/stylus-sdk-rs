// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use glob::glob;
use tiny_keccak::{Hasher, Keccak};

use super::{ProjectConfig, ProjectError};
use crate::{
    core::build::{BuildConfig, OptLevel},
    utils::{cargo, toolchain::find_toolchain_file},
};

pub type ProjectHash = [u8; 32];

pub fn hash_project(
    dir: impl AsRef<Path>,
    config: &ProjectConfig,
    build: &BuildConfig,
) -> Result<ProjectHash, ProjectError> {
    let cargo_version = cargo::version()?;

    let mut keccak = Keccak::v256();
    keccak.update(cargo_version.as_bytes());
    if matches!(build.opt_level, OptLevel::Z) {
        keccak.update(&[0]);
    } else {
        keccak.update(&[1]);
    }

    // Fetch the Rust toolchain toml file from the project root. Assert that it exists and add it to the
    // files in the directory to hash.
    let toolchain_file_path = find_toolchain_file(dir.as_ref())?;

    let mut paths = all_paths(dir, config.source_file_patterns.clone())?;
    paths.push(toolchain_file_path);
    paths.sort();

    // Read the file contents in another thread and process the keccak in the main thread.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for filename in paths.iter() {
            greyln!(
                "File used for deployment hash: {}",
                filename.as_os_str().to_string_lossy()
            );
            tx.send(read_file_preimage(filename))
                .expect("failed to send preimage (impossible)");
        }
    });
    for result in rx {
        keccak.update(result?.as_slice());
    }

    let mut project_hash = ProjectHash::default();
    keccak.finalize(&mut project_hash);
    greyln!(
        "project metadata hash computed on deployment: {:?}",
        hex::encode(project_hash)
    );
    Ok(project_hash)
}

fn all_paths(
    root_dir: impl AsRef<Path>,
    source_file_patterns: Vec<String>,
) -> Result<Vec<PathBuf>, ProjectError> {
    let mut files = Vec::<PathBuf>::new();
    let mut directories = Vec::<PathBuf>::new();
    directories.push(root_dir.as_ref().to_path_buf()); // Using `from` directly

    let glob_paths = expand_glob_patterns(source_file_patterns)?;

    while let Some(dir) = directories.pop() {
        for entry in fs::read_dir(&dir).map_err(|e| ProjectError::DirectoryRead(dir.clone(), e))? {
            let entry = entry.map_err(|e| ProjectError::DirectoryEntry(dir.clone(), e))?;
            let path = entry.path();

            if path.is_dir() {
                if path.ends_with("target") || path.ends_with(".git") {
                    continue; // Skip "target" and ".git" directories
                }
                directories.push(path);
            } else if path.file_name().is_some_and(|f| {
                // If the user has has specified a list of source file patterns, check if the file
                // matches the pattern.
                if !glob_paths.is_empty() {
                    for glob_path in glob_paths.iter() {
                        if glob_path == &path {
                            return true;
                        }
                    }
                    false
                } else {
                    // Otherwise, by default include all rust files, Cargo.toml and Cargo.lock files.
                    f == "Cargo.toml" || f == "Cargo.lock" || f.to_string_lossy().ends_with(".rs")
                }
            }) {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn expand_glob_patterns(patterns: Vec<String>) -> Result<Vec<PathBuf>, ProjectError> {
    let mut files_to_include = Vec::new();
    for pattern in patterns {
        let paths = glob(&pattern).map_err(|e| ProjectError::GlobPattern(pattern.clone(), e))?;
        for path_result in paths {
            let path = path_result?;
            files_to_include.push(path);
        }
    }
    Ok(files_to_include)
}

fn read_file_preimage(filename: &Path) -> Result<Vec<u8>, ProjectError> {
    let mut contents = Vec::with_capacity(1024);
    {
        let filename = filename.as_os_str();
        contents.extend_from_slice(&(filename.len() as u64).to_be_bytes());
        contents.extend_from_slice(filename.as_encoded_bytes());
    }
    let mut file = std::fs::File::open(filename)
        .map_err(|e| ProjectError::FileOpen(filename.to_path_buf(), e))?;
    contents.extend_from_slice(&file.metadata().unwrap().len().to_be_bytes());
    file.read_to_end(&mut contents)
        .map_err(|e| ProjectError::FileRead(filename.to_path_buf(), e))?;
    Ok(contents)
}
