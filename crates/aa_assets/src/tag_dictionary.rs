use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use aa_core::{project_path, SCHEMA_VERSION};

use crate::error::AssetError;

/// Supported schema version for `assets/data/tags.ron`.
pub const TAG_DICTIONARY_SCHEMA_VERSION: u32 = SCHEMA_VERSION;

/// Default relative path to the gameplay tag dictionary.
pub const TAG_DICTIONARY_PATH: &str = "assets/data/tags.ron";

/// Gameplay tag dictionary loaded from RON.
///
/// Matches the layout documented in `13_data_schemas.md`:
/// `schema_version` plus a flat list of dotted tag names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagDictionary {
    pub schema_version: u32,
    pub tags: Vec<String>,
}

impl TagDictionary {
    /// Validates the schema version and builds a lookup set for O(1) checks.
    pub fn validated(self) -> Result<Self, AssetError> {
        if self.schema_version != TAG_DICTIONARY_SCHEMA_VERSION {
            return Err(AssetError::UnsupportedSchemaVersion {
                found: self.schema_version,
                expected: TAG_DICTIONARY_SCHEMA_VERSION,
            });
        }
        Ok(self)
    }

    /// Returns a set view of all known tag names.
    pub fn tag_set(&self) -> HashSet<&str> {
        self.tags.iter().map(String::as_str).collect()
    }
}

impl Default for TagDictionary {
    fn default() -> Self {
        Self {
            schema_version: TAG_DICTIONARY_SCHEMA_VERSION,
            tags: Vec::new(),
        }
    }
}

/// Bevy resource holding the loaded tag dictionary.
#[derive(Resource, Debug, Default)]
pub struct TagDictionaryResource {
    dictionary: TagDictionary,
    loaded: bool,
}

impl TagDictionaryResource {
    /// Returns the loaded dictionary, if startup loading succeeded.
    pub fn get(&self) -> Option<&TagDictionary> {
        self.loaded.then_some(&self.dictionary)
    }

    /// Returns the dictionary, falling back to an empty default when not loaded.
    pub fn dictionary(&self) -> &TagDictionary {
        &self.dictionary
    }

    /// Returns `true` when a dictionary was loaded from disk.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn set_loaded(&mut self, dictionary: TagDictionary) {
        self.dictionary = dictionary;
        self.loaded = true;
    }
}

/// Loads `assets/data/tags.ron` when the file exists.
pub fn load_tag_dictionary(mut tag_dictionary: ResMut<TagDictionaryResource>) {
    let path = project_path(TAG_DICTIONARY_PATH);
    if !path.exists() {
        return;
    }

    match load_tag_dictionary_from_path(&path) {
        Ok(dictionary) => {
            tag_dictionary.set_loaded(dictionary);
        }
        Err(error) => {
            bevy::log::warn!("failed to load tag dictionary from {path:?}: {error}");
        }
    }
}

/// Parses and validates a tag dictionary from a RON file on disk.
pub fn load_tag_dictionary_from_path(path: &Path) -> Result<TagDictionary, AssetError> {
    let contents = std::fs::read_to_string(path)?;
    parse_tag_dictionary_ron(&contents)
}

fn parse_tag_dictionary_ron(contents: &str) -> Result<TagDictionary, AssetError> {
    #[derive(Deserialize)]
    enum TagDictionaryRon {
        TagDictionary {
            schema_version: u32,
            tags: Vec<String>,
        },
    }
    let wrapper: TagDictionaryRon = ron::from_str(contents)?;
    let TagDictionaryRon::TagDictionary {
        schema_version,
        tags,
    } = wrapper;
    TagDictionary {
        schema_version,
        tags,
    }
    .validated()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_tag_dictionary_ron() {
        let ron = r#"TagDictionary(
            schema_version: 1,
            tags: [
                "State.Stunned",
                "Damage.Fire",
            ],
        )"#;

        let dictionary =
            parse_tag_dictionary_ron(ron).expect("parse tags.ron");
        assert_eq!(dictionary.schema_version, 1);
        assert_eq!(dictionary.tags.len(), 2);
    }
}
