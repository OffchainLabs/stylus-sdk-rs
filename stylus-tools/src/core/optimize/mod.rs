// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Reproducible `wasm-opt` (Binaryen) post-build optimization.
//!
//! Large Stylus contracts may exceed the on-chain activation size limit unless the release Wasm
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

use std::{fs, path::Path, process::Command};

use cargo_metadata::MetadataCommand;
use serde::Deserialize;

use crate::{
    core::{manifest, project::contract::Contract},
    error::CommandFailure,
};

/// Name of the `wasm-opt` binary, expected on `PATH`.
pub const WASM_OPT_BIN: &str = "wasm-opt";

/// A resolved, ready-to-apply `wasm-opt` optimization recipe for a single contract.
///
/// This is the single source of truth used by both deploy and verification: applied inside the
/// Wasm processing funnel and folded into the `project_hash`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmOptConfig {
    /// Pinned Binaryen version, as a plain version number (e.g. `"131"`).
    pub version: String,
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

        let version = contract_table
            .version
            .or_else(|| workspace_table.as_ref().and_then(|w| w.version.clone()))
            .ok_or_else(|| WasmOptError::MissingVersion(contract.name().to_string()))?;
        if !is_valid_version(&version) {
            return Err(WasmOptError::InvalidVersion(version));
        }
        let flags = contract_table
            .flags
            .or_else(|| workspace_table.and_then(|w| w.flags))
            .unwrap_or_default();

        Ok(Some(Self { version, flags }))
    }
}

/// Optimize Wasm bytecode with the pinned `wasm-opt`, returning the optimized bytes.
///
/// The installed `wasm-opt` version is verified against the pinned version first, so a mismatched
/// local Binaryen fails loudly rather than silently producing non-reproducible bytes.
///
/// `wasm-opt` reads its input from a file and writes its output with `-o`; it does not support
/// streaming stdin/stdout reliably across versions, so the Wasm is round-tripped through a
/// temporary directory (cleaned up on drop).
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
pub fn verify_version(expected: &str) -> Result<(), WasmOptError> {
    let output = Command::new(WASM_OPT_BIN)
        .arg("--version")
        .output()
        .map_err(map_spawn_error)?;
    let stdout = CommandFailure::check(WASM_OPT_BIN, output)?;
    let found = parse_version(&stdout)
        .ok_or_else(|| WasmOptError::VersionParse(stdout.trim().to_string()))?;
    if found != expected {
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

/// A valid pinned version is a non-empty run of ASCII digits (e.g. `"131"`).
fn is_valid_version(version: &str) -> bool {
    !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit())
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
/// in with a bare table and inherit the workspace-root defaults.
#[derive(Debug, Deserialize)]
struct TomlWasmOpt {
    version: Option<String>,
    flags: Option<Vec<String>>,
}

/// Parse the `[wasm-opt]` table out of a `Stylus.toml`, if the file exists.
fn load_wasm_opt_table(path: &Path) -> Result<Option<TomlWasmOpt>, WasmOptError> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)?;
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
Run the reproducible build (without --no-verify) to have cargo-stylus install and use the pinned version automatically, \
or install Binaryen locally for a non-reproducible local build",
        bin = WASM_OPT_BIN
    )]
    NotFound,
    #[error("could not parse `wasm-opt --version` output: {0:?}")]
    VersionParse(String),
    #[error(
        "wasm-opt version mismatch: Stylus.toml pins version {expected} but the wasm-opt on PATH is version {found}. \
Run the reproducible build (without --no-verify) to have cargo-stylus install and use the pinned version automatically, \
or install Binaryen version {expected} locally"
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
        assert!(is_valid_version("117"));
        assert!(!is_valid_version(""));
        assert!(!is_valid_version("version_117"));
        assert!(!is_valid_version("117-rc1"));
    }
}
