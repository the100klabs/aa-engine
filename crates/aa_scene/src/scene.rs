use std::collections::HashMap;

use bevy::{asset::Asset, prelude::*, reflect::TypePath};
use serde::{Deserialize, Serialize};

/// Loaded scene asset with inline entities and optional prefab references.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct SceneAsset {
    pub schema_version: u32,
    pub id: String,
    pub entities: Vec<SceneEntity>,
}

/// One entity entry inside a scene RON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub components: HashMap<String, ron::Value>,
    #[serde(default)]
    pub prefab: Option<String>,
    #[serde(default)]
    pub children: Vec<SceneEntity>,
}

/// RON wrapper enum: `Scene(...)`.
#[derive(Debug, Deserialize)]
pub(crate) enum SceneRon {
    Scene(SceneAssetData),
}

#[derive(Debug, Deserialize)]
pub(crate) struct SceneAssetData {
    pub schema_version: u32,
    pub id: String,
    pub entities: Vec<SceneEntity>,
}

impl From<SceneAssetData> for SceneAsset {
    fn from(data: SceneAssetData) -> Self {
        Self {
            schema_version: data.schema_version,
            id: data.id,
            entities: data.entities,
        }
    }
}
