use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

/// Applies RON component maps from prefab/scene assets onto spawned entities.
pub(crate) fn apply_components(entity_commands: &mut EntityCommands, components: &HashMap<String, ron::Value>) {
    for (component_name, value) in components {
        match component_name.as_str() {
            "Transform" => {
                if let Some(transform) = deserialize_transform(value) {
                    entity_commands.insert(transform);
                } else {
                    warn!("failed to deserialize Transform from RON data");
                }
            }
            "Name" => {
                if let Some(name) = deserialize_name(value) {
                    entity_commands.insert(name);
                }
            }
            other => {
                warn!("unsupported RON component `{other}` — skipped in Phase 0");
            }
        }
    }
}

pub(crate) fn apply_entity_name(entity_commands: &mut EntityCommands, name: Option<&str>) {
    if let Some(name) = name {
        entity_commands.insert(Name::new(name.to_string()));
    }
}

#[derive(Debug, Deserialize)]
struct TransformRon {
    #[serde(default)]
    translation: (f32, f32, f32),
    #[serde(default = "default_rotation")]
    rotation: (f32, f32, f32, f32),
    #[serde(default = "default_scale")]
    scale: (f32, f32, f32),
}

fn default_rotation() -> (f32, f32, f32, f32) {
    (0.0, 0.0, 0.0, 1.0)
}

fn default_scale() -> (f32, f32, f32) {
    (1.0, 1.0, 1.0)
}

fn deserialize_transform(value: &ron::Value) -> Option<Transform> {
    let serialized = ron::ser::to_string(value).ok()?;
    let data: TransformRon = ron::de::from_str(&serialized).ok()?;
    Some(Transform {
        translation: Vec3::new(data.translation.0, data.translation.1, data.translation.2),
        rotation: Quat::from_xyzw(
            data.rotation.0,
            data.rotation.1,
            data.rotation.2,
            data.rotation.3,
        ),
        scale: Vec3::new(data.scale.0, data.scale.1, data.scale.2),
    })
}

fn deserialize_name(value: &ron::Value) -> Option<Name> {
    if let ron::Value::String(name) = value {
        return Some(Name::new(name.clone()));
    }
    None
}
