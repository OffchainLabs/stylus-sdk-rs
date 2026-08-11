// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Reproducible `wasm-opt` (Binaryen) post-build optimization.
//!
//! Large Stylus contracts may exceed the on-chain code size limit unless the release Wasm
//! is optimized with a pinned [Binaryen] `wasm-opt` before deployment. To keep such contracts
//! reproducibly verifiable, the exact optimization recipe (a pinned `wasm-opt` version and set of
//! flags) is declared in `Stylus.toml` and applied identically during both deploy and
//! verification, and is folded into the deployment `project_hash`.
//!
//! # Configuration (layered, opt-in)
//!
//! A contract is optimized only if its **own** `Stylus.toml` contains a `[wasm-opt]` table. The
//! workspace-root `Stylus.toml` may declare a `[wasm-opt]` table too, which supplies shared
//! default values (`version`/`flags`) that opted-in contracts inherit and may override. A bare
//! `[wasm-opt]` table in a contract means "opt in and inherit the workspace recipe". For a
//! single-contract project the lone `Stylus.toml` is both the workspace and the contract file, so
//! declaring `[wasm-opt]` there opts the contract in.
//!
//! ```toml
//! [wasm-opt]
//! version = "131"     # pinned Binaryen version
//! flags = ["-Oz"]     # passed verbatim to wasm-opt
//! ```
//!
//! [Binaryen]: https://github.com/WebAssembly/binaryen

use std::{fmt, fs, path::Path, process::Command, str::FromStr};

use cargo_metadata::MetadataCommand;
use serde::Deserialize;

use crate::{
    core::{manifest, project::contract::Contract},
    error::CommandFailure,
};

/// Name of the `wasm-opt` binary, expected on `PATH`.
pub const WASM_OPT_BIN: &str = "wasm-opt";

/// First cargo-stylus version whose baked-in base image understands the `[wasm-opt]` table. Older
/// binaries silently ignore it, so a reproducible build against them would deploy unoptimized bytes
/// while still verifying green.
pub const MIN_CARGO_STYLUS_VERSION: &str = "0.10.9";

/// A validated Binaryen version number (e.g. `"131"`): a non-empty run of ASCII digits with no
/// leading zeros.
///
/// Carrying the invariant in the type is what makes the version safe to interpolate into the
/// reproducible build's Dockerfile (no shell metacharacters) and into the Binaryen release URL
/// (no leading zeros, which would 404).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryenVersion(String);

impl BinaryenVersion {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for BinaryenVersion {
    type Err = WasmOptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let valid = !s.is_empty()
            && s.bytes().all(|b| b.is_ascii_digit())
            && (s.len() == 1 || !s.starts_with('0'));
        if valid {
            Ok(Self(s.to_string()))
        } else {
            Err(WasmOptError::InvalidVersion(s.to_string()))
        }
    }
}

impl fmt::Display for BinaryenVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A resolved, ready-to-apply `wasm-opt` optimization recipe for a single contract.
///
/// This is the single source of truth used by both deploy and verification: applied inside the
/// Wasm processing funnel and folded into the `project_hash`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmOptConfig {
    /// Pinned Binaryen version.
    pub version: BinaryenVersion,
    /// Flags passed verbatim to `wasm-opt` (e.g. `["-Oz"]`).
    pub flags: Vec<String>,
}

impl WasmOptConfig {
    /// Resolve the effective `wasm-opt` recipe for a contract, if it opted in.
    ///
    /// Returns `Ok(None)` when the contract's own `Stylus.toml` has no `[wasm-opt]` table (i.e. it
    /// did not opt in). Otherwise the contract-level table is merged over the workspace-root
    /// defaults (contract values win) and the pinned `version` must resolve to a valid version
    /// number.
    pub fn resolve_for_contract(contract: &Contract) -> Result<Option<Self>, WasmOptError> {
        let contract_manifest_dir = contract
            .package
            .manifest_path
            .parent()
            .ok_or(WasmOptError::ContractDir)?;
        let contract_toml = contract_manifest_dir.join(manifest::FILENAME);

        // Opt-in is signalled by the presence of a [wasm-opt] table in the contract's own file.
        let Some(contract_table) = load_wasm_opt_table(contract_toml.as_std_path())? else {
            return Ok(None);
        };

        // Fetch the workspace-root defaults (skip if it is the same file as the contract's).
        let workspace_root = MetadataCommand::new().no_deps().exec()?.workspace_root;
        let workspace_toml = workspace_root.join(manifest::FILENAME);
        let workspace_table = if workspace_toml == contract_toml {
            None
        } else {
            load_wasm_opt_table(workspace_toml.as_std_path())?
        };

        let config = merge_tables(contract.name(), contract_table, workspace_table)?;
        if config.flags.is_empty() {
            warn!(@yellow,
                "[wasm-opt] is enabled for contract '{}' but no flags are set, so wasm-opt will run no optimization passes; add flags (e.g. [\"-Oz\"]) or remove the [wasm-opt] table",
                contract.name()
            );
        }
        Ok(Some(config))
    }
}

/// Merge a contract's `[wasm-opt]` table over the workspace-root defaults (contract values win),
/// yielding the resolved recipe.
fn merge_tables(
    contract_name: &str,
    contract: TomlWasmOpt,
    workspace: Option<TomlWasmOpt>,
) -> Result<WasmOptConfig, WasmOptError> {
    let version = contract
        .version
        .or_else(|| workspace.as_ref().and_then(|w| w.version.clone()))
        .ok_or_else(|| WasmOptError::MissingVersion(contract_name.to_string()))?
        .parse::<BinaryenVersion>()?;
    let flags = contract
        .flags
        .or_else(|| workspace.and_then(|w| w.flags))
        .unwrap_or_default();
    Ok(WasmOptConfig { version, flags })
}

/// Optimize Wasm bytecode with the pinned `wasm-opt`, returning the optimized bytes.
///
/// The installed `wasm-opt` version is verified against the pinned version first, so a mismatched
/// local Binaryen fails loudly rather than silently producing non-reproducible bytes.
///
/// `wasm-opt` reads its input from a file and writes its output with `-o`, so the Wasm is
/// round-tripped through a temporary directory (cleaned up on drop).
pub fn optimize(wasm: &[u8], config: &WasmOptConfig) -> Result<Vec<u8>, WasmOptError> {
    verify_version(&config.version)?;

    let dir = tempfile::tempdir()?;
    let input = dir.path().join("input.wasm");
    let output = dir.path().join("output.wasm");
    fs::write(&input, wasm)?;

    info!(@grey, "Optimizing Wasm with wasm-opt {} {}", config.version, config.flags.join(" "));

    let cmd_output = Command::new(WASM_OPT_BIN)
        .args(&config.flags)
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .output()
        .map_err(map_spawn_error)?;
    CommandFailure::check(WASM_OPT_BIN, cmd_output)?;

    Ok(fs::read(&output)?)
}

/// Verify that the `wasm-opt` on `PATH` matches the pinned version.
pub fn verify_version(expected: &BinaryenVersion) -> Result<(), WasmOptError> {
    let output = Command::new(WASM_OPT_BIN)
        .arg("--version")
        .output()
        .map_err(map_spawn_error)?;
    let stdout = CommandFailure::check(WASM_OPT_BIN, output)?;
    check_installed_version(expected, &stdout)
}

/// Compare the pinned version against `wasm-opt --version` output.
fn check_installed_version(
    expected: &BinaryenVersion,
    version_output: &str,
) -> Result<(), WasmOptError> {
    let found = parse_version(version_output)
        .ok_or_else(|| WasmOptError::VersionParse(version_output.trim().to_string()))?;
    if found != expected.as_str() {
        return Err(WasmOptError::VersionMismatch {
            expected: expected.to_string(),
            found,
        });
    }
    Ok(())
}

/// Parse the numeric version from `wasm-opt --version` output.
///
/// Binaryen prints e.g. `wasm-opt version 117` (some builds append a git suffix), so we take the
/// leading digits of the token following `version`.
fn parse_version(output: &str) -> Option<String> {
    let mut tokens = output.split_whitespace();
    tokens.by_ref().position(|t| t == "version")?;
    let version_token = tokens.next()?;
    let digits: String = version_token
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    (!digits.is_empty()).then_some(digits)
}

fn map_spawn_error(e: std::io::Error) -> WasmOptError {
    if e.kind() == std::io::ErrorKind::NotFound {
        WasmOptError::NotFound
    } else {
        WasmOptError::Io(e)
    }
}

/// Minimal view over a `Stylus.toml` used only to extract its `[wasm-opt]` table.
#[derive(Debug, Deserialize)]
struct WasmOptManifest {
    #[serde(default, rename = "wasm-opt")]
    wasm_opt: Option<TomlWasmOpt>,
}

/// Serialized form of the `[wasm-opt]` table. All fields are optional so that a contract may opt
/// in with a bare table and inherit the workspace-root defaults. Unknown keys are rejected so a
/// typo like `flag = [...]` fails loudly instead of silently resolving to zero flags.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlWasmOpt {
    version: Option<String>,
    flags: Option<Vec<String>>,
}

/// Parse the `[wasm-opt]` table out of a `Stylus.toml`, if the file exists.
///
/// Only `NotFound` maps to "no manifest" — a `Path::exists()` pre-check would swallow other
/// errors (e.g. permission denied), silently reading an opted-in contract as not opted in.
fn load_wasm_opt_table(path: &Path) -> Result<Option<TomlWasmOpt>, WasmOptError> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let manifest: WasmOptManifest = toml::from_str(&contents)?;
    Ok(manifest.wasm_opt)
}

#[derive(Debug, thiserror::Error)]
pub enum WasmOptError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error("failed to parse Stylus.toml: {0}")]
    TomlRead(#[from] toml::de::Error),
    #[error("{0}")]
    CommandFailure(#[from] CommandFailure),

    #[error("could not determine contract directory")]
    ContractDir,
    #[error(
        "[wasm-opt] is enabled for contract '{0}' but no `version` is set in its Stylus.toml or the workspace Stylus.toml"
    )]
    MissingVersion(String),
    #[error(
        "invalid wasm-opt `version` {0:?}: expected a Binaryen version number such as \"131\""
    )]
    InvalidVersion(String),

    #[error(
        "`{bin}` (Binaryen) not found on PATH, but a [wasm-opt] table in Stylus.toml requests it. \
Install the pinned Binaryen version locally, or for `deploy`/`verify` run the reproducible build \
(without --no-verify), which installs and uses the pinned version automatically",
        bin = WASM_OPT_BIN
    )]
    NotFound,
    #[error("could not parse `wasm-opt --version` output: {0:?}")]
    VersionParse(String),
    #[error(
        "wasm-opt version mismatch: Stylus.toml pins version {expected} but the wasm-opt on PATH is version {found}. \
Install Binaryen version {expected} locally, or for `deploy`/`verify` run the reproducible build \
(without --no-verify), which installs and uses the pinned version automatically"
    )]
    VersionMismatch { expected: String, found: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_version() {
        assert_eq!(
            parse_version("wasm-opt version 117").as_deref(),
            Some("117")
        );
    }

    #[test]
    fn parses_version_with_git_suffix() {
        assert_eq!(
            parse_version("wasm-opt version 116 (version_116-8-gabc123)").as_deref(),
            Some("116")
        );
    }

    #[test]
    fn rejects_missing_version_token() {
        assert_eq!(parse_version("garbage output"), None);
    }

    #[test]
    fn validates_version_numbers() {
        assert!("117".parse::<BinaryenVersion>().is_ok());
        assert!("0".parse::<BinaryenVersion>().is_ok());
        assert!("".parse::<BinaryenVersion>().is_err());
        assert!("version_117".parse::<BinaryenVersion>().is_err());
        assert!("117-rc1".parse::<BinaryenVersion>().is_err());
        // Leading zeros would 404 in the Binaryen release URL.
        assert!("0131".parse::<BinaryenVersion>().is_err());
    }

    fn table(version: Option<&str>, flags: Option<&[&str]>) -> TomlWasmOpt {
        TomlWasmOpt {
            version: version.map(str::to_string),
            flags: flags.map(|f| f.iter().map(|s| s.to_string()).collect()),
        }
    }

    #[test]
    fn merge_contract_only() {
        let resolved = merge_tables("c", table(Some("131"), Some(&["-Oz"])), None).unwrap();
        assert_eq!(resolved.version.as_str(), "131");
        assert_eq!(resolved.flags, vec!["-Oz".to_string()]);
    }

    #[test]
    fn merge_inherits_workspace_defaults() {
        let workspace = table(Some("131"), Some(&["-Oz"]));
        let resolved = merge_tables("c", table(None, None), Some(workspace)).unwrap();
        assert_eq!(resolved.version.as_str(), "131");
        assert_eq!(resolved.flags, vec!["-Oz".to_string()]);
    }

    #[test]
    fn merge_contract_overrides_workspace() {
        let workspace = table(Some("116"), Some(&["-O2"]));
        let contract = table(Some("131"), Some(&["-Oz"]));
        let resolved = merge_tables("c", contract, Some(workspace)).unwrap();
        assert_eq!(resolved.version.as_str(), "131");
        assert_eq!(resolved.flags, vec!["-Oz".to_string()]);
    }

    #[test]
    fn merge_mixes_version_and_flags_across_levels() {
        let workspace = table(Some("131"), Some(&["-O2"]));
        let contract = table(None, Some(&["-Oz"]));
        let resolved = merge_tables("c", contract, Some(workspace)).unwrap();
        assert_eq!(resolved.version.as_str(), "131");
        assert_eq!(resolved.flags, vec!["-Oz".to_string()]);
    }

    #[test]
    fn merge_missing_version_errors() {
        let err = merge_tables("mycontract", table(None, Some(&["-Oz"])), None).unwrap_err();
        assert!(matches!(err, WasmOptError::MissingVersion(name) if name == "mycontract"));
    }

    #[test]
    fn merge_invalid_version_errors() {
        let err = merge_tables("c", table(Some("1.31"), None), None).unwrap_err();
        assert!(matches!(err, WasmOptError::InvalidVersion(v) if v == "1.31"));
    }

    #[test]
    fn check_installed_version_matches() {
        assert!(check_installed_version(&"117".parse().unwrap(), "wasm-opt version 117").is_ok());
    }

    #[test]
    fn check_installed_version_mismatch() {
        let err =
            check_installed_version(&"131".parse().unwrap(), "wasm-opt version 117").unwrap_err();
        assert!(matches!(
            err,
            WasmOptError::VersionMismatch { expected, found } if expected == "131" && found == "117"
        ));
    }

    #[test]
    fn check_installed_version_unparseable() {
        let err = check_installed_version(&"117".parse().unwrap(), "garbage output").unwrap_err();
        assert!(matches!(err, WasmOptError::VersionParse(_)));
    }

    /// Write `contents` as a Stylus.toml in a fresh tempdir and load its [wasm-opt] table.
    fn load_from_str(contents: &str) -> Result<Option<TomlWasmOpt>, WasmOptError> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(manifest::FILENAME);
        fs::write(&path, contents).unwrap();
        load_wasm_opt_table(&path)
    }

    #[test]
    fn loads_wasm_opt_table() {
        let table = load_from_str(
            r#"
            [contract]

            [wasm-opt]
            version = "131"
            flags = ["-Oz"]
            "#,
        )
        .unwrap()
        .expect("table should be present");
        assert_eq!(table.version.as_deref(), Some("131"));
        assert_eq!(table.flags, Some(vec!["-Oz".to_string()]));
    }

    #[test]
    fn load_without_table_is_none() {
        assert!(load_from_str("[contract]\n").unwrap().is_none());
    }

    #[test]
    fn load_missing_file_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join(manifest::FILENAME);
        assert!(load_wasm_opt_table(&missing).unwrap().is_none());
    }

    #[test]
    fn load_rejects_unknown_keys() {
        // A typo inside the table must fail loudly, not resolve to a defaulted recipe.
        let err = load_from_str("[wasm-opt]\nflag = [\"-Oz\"]\n").unwrap_err();
        assert!(matches!(err, WasmOptError::TomlRead(_)));
    }
}
