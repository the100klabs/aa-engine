use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use aa_core::SCHEMA_VERSION;

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "ExperienceDefinition")]
pub struct ExperienceDefinitionAsset {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub game_features: Vec<String>,
    pub default_pawn: String,
    #[serde(default)]
    pub action_sets: Vec<String>,
    pub actions: Vec<ExperienceAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceAction {
    GrantAbilitySet { abilities: Vec<String> },
    AddInputContext { context: String },
    LoadAttributeSet { path: String },
}

pub fn validate_schema_version(found: u32) -> Result<(), u32> {
    if found != SCHEMA_VERSION {
        Err(found)
    } else {
        Ok(())
    }
}
