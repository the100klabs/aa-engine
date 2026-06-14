use std::collections::HashMap;

use bevy::{asset::Asset, prelude::*, reflect::TypePath};
use serde::{Deserialize, Serialize};

/// Loaded prefab asset matching `schemas/prefab.schema.json`.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct PrefabAsset {
    pub schema_version: u32,
    pub id: String,
    pub children: Vec<PrefabEntity>,
}

/// One node in a prefab hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabEntity {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub components: HashMap<String, ron::Value>,
    #[serde(default)]
    pub children: Vec<PrefabEntity>,
}

/// RON on-disk shape: `Prefab(...)`.
#[derive(Debug, Deserialize)]
#[serde(rename = "Prefab")]
pub(crate) struct PrefabAssetData {
    pub schema_version: u32,
    pub id: String,
    pub children: Vec<PrefabEntity>,
}

impl From<PrefabAssetData> for PrefabAsset {
    fn from(data: PrefabAssetData) -> Self {
        Self {
            schema_version: data.schema_version,
            id: data.id,
            children: data.children,
        }
    }
}
