use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use aa_core::SCHEMA_VERSION;

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "AttributeSet")]
pub struct AttributeSetAsset {
    pub schema_version: u32,
    pub id: String,
    pub attributes: Vec<AttributeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDef {
    pub name: String,
    pub default: f32,
    #[serde(default)]
    pub min: f32,
    #[serde(default = "default_max")]
    pub max: f32,
}

fn default_max() -> f32 {
    f32::MAX
}

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "GameplayEffect")]
pub struct GameplayEffectAsset {
    pub schema_version: u32,
    pub id: String,
    pub duration: EffectDuration,
    pub modifiers: Vec<EffectModifier>,
    #[serde(default)]
    pub granted_tags: Vec<String>,
    #[serde(default)]
    pub application_tags_required: Vec<String>,
    #[serde(default)]
    pub application_tags_blocked: Vec<String>,
    #[serde(default)]
    pub cues_on_apply: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectDuration {
    Instant,
    Infinite,
    Periodic { seconds: f32, count: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectModifier {
    pub attribute: String,
    pub op: ModifierOp,
    pub magnitude: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModifierOp {
    Add,
    Multiply,
    Override,
}

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "GameplayAbility")]
pub struct GameplayAbilityAsset {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub cooldown_tags: Vec<String>,
    #[serde(default)]
    pub activation_tags_required: Vec<String>,
    #[serde(default)]
    pub activation_tags_blocked: Vec<String>,
    pub cost_effect: Option<String>,
    pub montage: Option<String>,
    pub cue_on_activate: Option<String>,
    pub r#impl: String,
}

/// Validates schema version after RON parse.
pub fn validate_schema_version(found: u32) -> Result<(), u32> {
    if found != SCHEMA_VERSION {
        Err(found)
    } else {
        Ok(())
    }
}
