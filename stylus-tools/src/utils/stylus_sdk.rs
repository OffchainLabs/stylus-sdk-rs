// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities relating to the stylus-sdk itself.

use std::path::Path;

use super::cargo::DepSource;

/// Version requirements for contract dependencies that must be compatible with the stylus-sdk.
/// These are updated as part of the release process alongside workspace dependency versions.
const ALLOY_PRIMITIVES_VERSION: &str = "1.5.7";
const ALLOY_SOL_TYPES_VERSION: &str = "1.5.7";

/// Required dependencies for stylus contracts.
///
/// These dependencies use version requirements that are kept in sync with the stylus-sdk crate
/// to ensure ABI type compatibility.
///
/// If `sdk_path` is provided, stylus-sdk will be added as a path dependency instead of a
/// versioned registry dependency.
pub fn contract_dependencies(
    sdk_path: Option<&Path>,
) -> impl IntoIterator<Item = (String, DepSource)> {
    let sdk_dep = match sdk_path {
        Some(path) => DepSource::Path(path.to_path_buf()),
        None => DepSource::Version(env!("CARGO_PKG_VERSION").to_string()),
    };
    [
        (
            "alloy-primitives".to_string(),
            DepSource::Version(ALLOY_PRIMITIVES_VERSION.to_string()),
        ),
        (
            "alloy-sol-types".to_string(),
            DepSource::Version(ALLOY_SOL_TYPES_VERSION.to_string()),
        ),
        ("stylus-sdk".to_string(), sdk_dep),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_dependencies() {
        for (name, source) in contract_dependencies(None) {
            match source {
                DepSource::Version(v) => match name.as_str() {
                    "alloy-primitives" | "alloy-sol-types" => assert!(v.starts_with("1")),
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
        let has_sdk_path_dep = contract_dependencies(Some(sdk_path))
            .into_iter()
            .any(|(name, source)| {
                name == "stylus-sdk" && matches!(source, DepSource::Path(p) if p == sdk_path)
            });
        assert!(has_sdk_path_dep, "expected path dep for stylus-sdk");
    }
}
