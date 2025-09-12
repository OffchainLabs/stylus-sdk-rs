// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities for modifying cargo manifest files (`Cargo.toml`).

use std::{collections::BTreeSet, fs, path::PathBuf};

use toml_edit::{Array, DocumentMut, Entry, Item, Table, Value};

/// Make modifications to a `Cargo.toml` file.
#[derive(Debug)]
pub struct ManifestMut {
    path: PathBuf,
    doc: DocumentMut,
}

impl ManifestMut {
    /// Read the `Cargo.toml` file from a specific path.
    ///
    /// The path is stored as well for simple writing after it is edited.
    pub fn read(path: impl Into<PathBuf>) -> Result<Self, CargoManifestError> {
        let path = path.into();
        let doc = fs::read_to_string(&path)?.parse()?;
        Ok(ManifestMut { path, doc })
    }

    /// Write all modifications back to the `Cargo.toml` file.
    pub fn write(&self) -> Result<(), CargoManifestError> {
        fs::write(&self.path, self.doc.to_string())?;
        Ok(())
    }

    /// Make modifications to the `[lib]` table.
    pub fn lib(&mut self) -> Lib<'_> {
        let entry = self.doc.entry("lib");
        let item = entry.or_insert_with(|| Table::new().into());
        Lib { item }
    }

    pub fn features(&mut self) -> Features<'_> {
        let entry = self.doc.entry("features");
        let item = entry.or_insert_with(|| Table::new().into());
        Features { item }
    }

    pub fn profile(&mut self, name: &str) -> Result<Profile<'_>, CargoManifestError> {
        // Create the profile table if it does not exist
        let profile_table = self
            .doc
            .entry("profile")
            .or_insert_with(|| Table::new().into())
            .as_table_mut()
            .ok_or(CargoManifestError::Invalid)?;
        // Do not display a `[profile]` table if it is empty (other than inner tables)
        profile_table.set_implicit(true);
        // Create the named profile table (`[profile.release]` for example) which will contain the profile info
        let table = profile_table
            .entry(name)
            .or_insert_with(|| Table::new().into())
            .as_table_mut()
            .ok_or(CargoManifestError::Invalid)?;
        Ok(Profile { table })
    }
}

#[derive(Debug)]
pub struct Features<'a> {
    item: &'a mut Item,
}

impl Features<'_> {
    pub fn extend_feature<'a>(
        &mut self,
        feature: &str,
        deps: impl IntoIterator<Item = &'a str>,
    ) -> Result<&mut Self, CargoManifestError> {
        let array = get_or_create_array(self.item, feature)?;
        array_extend_unique_strs(array, deps);
        Ok(self)
    }
}

/// The `[lib]` table in a cargo manifest.
#[derive(Debug)]
pub struct Lib<'a> {
    item: &'a mut Item,
}

impl Lib<'_> {
    /// Add to the `crate-type` key in the `[lib]` table.
    pub fn extend_crate_type<'a>(
        &mut self,
        items: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), CargoManifestError> {
        let array = get_or_create_array(self.item, "crate-type")?;
        array_extend_unique_strs(array, items);
        Ok(())
    }
}

/// Get or create an array inside a given table.
///
/// If the array exists, it is extended with the given items, checking for uniqueness. Otherwise,
/// the array will bei initialized with the given items.
fn get_or_create_array<'a>(
    table_item: &'a mut Item,
    key: &'a str,
) -> Result<&'a mut Array, CargoManifestError> {
    let array = table_item
        .as_table_mut()
        .ok_or(CargoManifestError::Invalid)?
        .entry(key)
        .or_insert_with(|| Array::new().into())
        .as_array_mut()
        .ok_or(CargoManifestError::Invalid)?;
    Ok(array)
}

/// Add unique strings to an array.
///
/// Note that the uniqueness of input strings is only checked against the initial values within the
/// array. Duplicate input strings will not be detected.
fn array_extend_unique_strs<'a>(array: &mut Array, new_strs: impl IntoIterator<Item = &'a str>) {
    // Create set of current string values, so we do not add duplicates
    let existing_strs = array
        .iter()
        .filter_map(|value| value.as_str().map(String::from))
        .collect::<BTreeSet<_>>();
    // Filter input strings
    let new_strs = new_strs
        .into_iter()
        .filter(|new_str| !existing_strs.contains(*new_str));
    // Add filtered strings to the array
    array.extend(new_strs);
}

/// A specific profile table in a cargo manifest (`[profile.release]` for example).
#[derive(Debug)]
pub struct Profile<'a> {
    table: &'a mut Table,
}

impl Profile<'_> {
    /// Set a default key, value pair within the profile table.
    ///
    /// Returns true if a new key/value pair was inserted.
    ///
    /// No modification is made if the key is already set.
    pub fn set_default(&mut self, key: &str, value: impl Into<Value>) -> bool {
        let entry = self.table.entry(key);
        let inserted = matches!(entry, Entry::Vacant(_));
        entry.or_insert_with(|| Item::from(value));
        inserted
    }

    /// Add a comment string above some specific key.
    pub fn add_comment(&mut self, key: &str, comment: &str) -> Result<(), CargoManifestError> {
        self.table
            .key_mut(key)
            .ok_or(CargoManifestError::Invalid)?
            .leaf_decor_mut()
            .set_prefix(comment);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CargoManifestError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml edit error: {0}")]
    TomlEdit(#[from] toml_edit::TomlError),

    // TODO: could be more informative
    #[error("invalid cargo manifest")]
    Invalid,
}
