use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::AssetError;
use crate::registry::AssetRegistry;
use aa_core::{project_path, SCHEMA_VERSION};

/// Supported schema version for `assets/asset_manifest.json`.
pub const ASSET_MANIFEST_SCHEMA_VERSION: u32 = SCHEMA_VERSION;

/// Default relative path to the generated asset manifest.
pub const ASSET_MANIFEST_PATH: &str = "assets/asset_manifest.json";

/// Generated manifest listing known assets, hashes, and dependency edges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifest {
    pub schema_version: u32,
    pub generated_at: String,
    pub assets: Vec<AssetManifestEntry>,
}

/// Single manifest row for one asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifestEntry {
    pub id: String,
    pub kind: String,
    pub hash: String,
    pub deps: Vec<String>,
}

impl AssetManifest {
    /// Validates the schema version before use.
    pub fn validated(self) -> Result<Self, AssetError> {
        if self.schema_version != ASSET_MANIFEST_SCHEMA_VERSION {
            return Err(AssetError::UnsupportedSchemaVersion {
                found: self.schema_version,
                expected: ASSET_MANIFEST_SCHEMA_VERSION,
            });
        }
        Ok(self)
    }
}

/// Startup stub: loads `assets/asset_manifest.json` when present and seeds
/// [`AssetRegistry`] with manifest asset ids.
pub fn load_asset_manifest(mut registry: ResMut<AssetRegistry>) {
    let path = project_path(ASSET_MANIFEST_PATH);
    if !path.exists() {
        return;
    }

    match load_asset_manifest_from_path(&path) {
        Ok(manifest) => {
            for entry in manifest.assets {
                registry.register(entry.id);
            }
        }
        Err(error) => {
            bevy::log::warn!("failed to load asset manifest from {path:?}: {error}");
        }
    }
}

/// Parses and validates an asset manifest from JSON on disk.
pub fn load_asset_manifest_from_path(path: &Path) -> Result<AssetManifest, AssetError> {
    let contents = std::fs::read_to_string(path)?;
    let manifest: AssetManifest = serde_json::from_str(&contents)?;
    manifest.validated()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_asset_manifest_json() {
        let json = r#"{
            "schema_version": 1,
            "generated_at": "2026-06-13T12:00:00Z",
            "assets": [
                {
                    "id": "abilities/fireball.ron",
                    "kind": "gameplay_ability",
                    "hash": "sha256:def456",
                    "deps": ["effects/burning.ron"]
                }
            ]
        }"#;

        let manifest: AssetManifest = serde_json::from_str(json).expect("parse manifest");
        assert_eq!(manifest.assets.len(), 1);
        assert_eq!(manifest.assets[0].id, "abilities/fireball.ron");
    }
}
