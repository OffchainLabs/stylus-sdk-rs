// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities relating to the stylus-sdk itself.

use std::path::Path;

use alloy::primitives::map::HashMap;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use cargo_util_schemas::manifest::PackageName;

use super::cargo::DepSource;

static CONTRACT_DEPENDENCIES: &[&str] = &["alloy-primitives", "alloy-sol-types"];

/// Required dependencies for stylus contracts.
///
/// These dependencies will use the version requirements specified for the stylus-sdk crate. This
/// is important to ensure sdk compatibility when those types are used within the contract.
///
/// If `sdk_path` is provided, stylus-sdk will be added as a path dependency instead of a
/// versioned registry dependency.
pub fn contract_dependencies(
    sdk_path: Option<&Path>,
) -> Result<HashMap<String, DepSource>, cargo_metadata::Error> {
    let sdk_dep = match sdk_path {
        Some(path) => DepSource::Path(path.to_path_buf()),
        None => DepSource::Version(env!("CARGO_PKG_VERSION").to_string()),
    };
    let deps = stylus_sdk_dependencies()?
        .filter(|(name, _req)| CONTRACT_DEPENDENCIES.contains(&name.as_str()))
        .map(|(name, req)| (name, DepSource::Version(req)))
        .chain(std::iter::once(("stylus-sdk".to_string(), sdk_dep)))
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
        let deps = contract_dependencies(None).unwrap();
        for (name, source) in &deps {
            match source {
                DepSource::Version(v) => match name.as_str() {
                    "alloy-primitives" | "alloy-sol-types" => assert!(v.starts_with("^1")),
                    "stylus-sdk" => assert_eq!(v, env!("CARGO_PKG_VERSION")),
                    other => panic!("unexpected dep: {other}"),
                },
                DepSource::Path(_) => panic!("unexpected path dep with sdk_path=None"),
            }
        }
    }

    #[test]
    fn test_contract_dependencies_with_sdk_path() {
        let sdk_path = std::path::Path::new("/tmp/fake-sdk");
        let deps = contract_dependencies(Some(sdk_path)).unwrap();
        match &deps["stylus-sdk"] {
            DepSource::Path(p) => assert_eq!(p, sdk_path),
            DepSource::Version(_) => panic!("expected path dep for stylus-sdk"),
        }
    }
}
