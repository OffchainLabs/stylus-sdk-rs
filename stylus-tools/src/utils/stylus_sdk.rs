// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities relating to the stylus-sdk itself.

use std::iter;

use alloy::primitives::map::HashMap;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use cargo_util_schemas::manifest::PackageName;

static CONTRACT_DEPENDENCIES: &[&str] = &["alloy-primitives", "alloy-sol-types"];

/// Required dependencies for stylus contracts.
///
/// These dependencies will use the version requirements specified for the stylus-sdk crate. This
/// is important to ensure sdk compatibility when those types are used within the contract.
pub fn contract_dependencies() -> Result<HashMap<String, String>, cargo_metadata::Error> {
    let deps = stylus_sdk_dependencies()?
        .filter(|(name, _req)| CONTRACT_DEPENDENCIES.contains(&name.as_str()))
        .chain(iter::once((
            "stylus-sdk".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )))
        .collect();
    Ok(deps)
}

/// Get all dependencies for stylus-sdk.
///
/// We can use these to determine required dependencies for contracts using the
/// [contract_dependencies] function.
fn stylus_sdk_dependencies() -> Result<impl Iterator<Item = (String, String)>, cargo_metadata::Error>
{
    let package = stylus_sdk_metadata()?;
    let deps = package
        .dependencies
        .into_iter()
        .map(|dep| (dep.name, dep.req.to_string()));
    Ok(deps)
}

/// Get the package metadata for the stylus-sdk crate.
fn stylus_sdk_metadata() -> Result<Package, cargo_metadata::Error> {
    let workspace = workspace_metadata()?;
    let package_name = PackageName::new("stylus-sdk".to_string()).expect("PackageName validation");
    let package = workspace
        .packages
        .into_iter()
        .find(|p| p.name == package_name)
        .expect("Finding stylus-sdk package");
    Ok(package)
}

/// Get metadata for the stylus-sdk-rs workspace.
fn workspace_metadata() -> Result<Metadata, cargo_metadata::Error> {
    let metadata = MetadataCommand::new()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .exec()?;
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_dependencies() {
        let deps = contract_dependencies().unwrap();
        assert!(deps["alloy-primitives"].starts_with("^1"));
        assert!(deps["alloy-sol-types"].starts_with("^1"));
        assert_eq!(deps["stylus-sdk"], env!("CARGO_PKG_VERSION"));
    }
}
