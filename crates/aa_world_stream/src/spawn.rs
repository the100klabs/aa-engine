use std::path::Path;

use bevy::prelude::*;
use serde::Deserialize;

use crate::assets::DataLayerState;
use crate::registry::{SectorRuntimeState, StreamedSectorEntity};

/// Marks a pawn spawned from a sector spawn table (cleaned up on sector deactivation).
#[derive(Component, Debug, Clone)]
pub struct SpawnedPawn {
    pub sector_id: String,
    pub entry_id: String,
    pub stable_name: String,
}

/// Stub AI profile attachment for camp guards (OWA-07).
#[derive(Component, Debug, Clone)]
pub struct CampGuardAi {
    pub profile_path: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SpawnTableBudgets {
    #[serde(default)]
    pub max_alive: u32,
    #[serde(default = "default_max_spawn")]
    pub max_spawn_per_activation: u32,
    #[serde(default)]
    pub memory_budget_mb: f32,
}

fn default_max_spawn() -> u32 {
    8
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpawnTableEntry {
    pub id: String,
    pub pawn: String,
    #[serde(default)]
    pub ai_profile: Option<String>,
    #[serde(default)]
    pub prefab: Option<String>,
    #[serde(default)]
    pub weight: f32,
    #[serde(default = "default_count_min")]
    pub count_min: u32,
    #[serde(default = "default_count_max")]
    pub count_max: u32,
    #[serde(default)]
    pub fixed_positions: Vec<[f32; 3]>,
}

fn default_count_min() -> u32 {
    1
}

fn default_count_max() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpawnTableAssetData {
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub data_layers: Vec<String>,
    pub entries: Vec<SpawnTableEntry>,
    #[serde(default)]
    pub budgets: SpawnTableBudgets,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct SpawnTableAsset {
    pub schema_version: u32,
    pub id: String,
    pub data_layers: Vec<String>,
    pub entries: Vec<SpawnTableEntry>,
    pub budgets: SpawnTableBudgets,
}

impl From<SpawnTableAssetData> for SpawnTableAsset {
    fn from(data: SpawnTableAssetData) -> Self {
        Self {
            schema_version: data.schema_version,
            id: data.id,
            data_layers: data.data_layers,
            entries: data.entries,
            budgets: data.budgets,
        }
    }
}

/// Returns true when a sector entity prefab path references a spawn table asset.
pub fn is_spawn_table_ref(prefab: &str) -> bool {
    prefab.contains("spawn_tables/") && prefab.ends_with(".ron")
}

/// Loads a spawn table RON file from disk (CLI + synchronous activation path).
pub fn load_spawn_table_from_disk(project_root: &Path, path: &str) -> Result<SpawnTableAsset, String> {
    let full = crate::registry::resolve_asset_path_for_disk(project_root, path);
    let text = std::fs::read_to_string(&full).map_err(|e| format!("failed to read {}: {e}", full.display()))?;
    let data: SpawnTableAssetData =
        ron::from_str(&text).map_err(|e| format!("invalid spawn table RON {}: {e}", full.display()))?;
    Ok(SpawnTableAsset::from(data))
}

fn encounters_layer_active(state: &SectorRuntimeState) -> bool {
    state
        .layer_states
        .get("encounters")
        .is_none_or(|layer| matches!(layer, DataLayerState::Active | DataLayerState::Loaded))
}

/// Spawns pawns from spawn-table refs in an active sector; returns spawned entity ids.
#[allow(clippy::too_many_arguments)]
pub fn activate_sector_spawns(
    commands: &mut Commands,
    project_root: &Path,
    sector_id: &str,
    sector_entities: &[crate::assets::SectorEntityPlacement],
    state: &mut SectorRuntimeState,
) {
    if !encounters_layer_active(state) {
        return;
    }

    let max_spawn: u32 = if state.ref_descriptor.id.contains("sector_0_0") {
        8
    } else {
        4
    };
    let mut spawned_count = 0u32;

    for placement in sector_entities {
        if !is_spawn_table_ref(&placement.prefab) {
            continue;
        }
        let Ok(table) = load_spawn_table_from_disk(project_root, &placement.prefab) else {
            continue;
        };

        let table_layers_ok = table.data_layers.is_empty()
            || table
                .data_layers
                .iter()
                .all(|layer| state.layer_states.get(layer).is_none_or(|s| {
                    matches!(s, DataLayerState::Active | DataLayerState::Loaded)
                }));
        if !table_layers_ok {
            continue;
        }

        let base_rotation = Quat::from_rotation_y(placement.transform.rotation_y_degrees.to_radians());
        let base_translation = Vec3::from_array(placement.transform.translation);

        for entry in &table.entries {
            let limit = entry
                .count_max
                .min(table.budgets.max_spawn_per_activation)
                .min(max_spawn.saturating_sub(spawned_count));
            for (index, position) in entry.fixed_positions.iter().enumerate() {
                if spawned_count >= limit {
                    break;
                }
                let stable_name = if index == 0 {
                    entry.id.clone()
                } else {
                    format!("{}_{index}", entry.id)
                };
                let local = Vec3::from_array(*position);
                let world_pos = base_translation + base_rotation * local;
                let mut cmd = commands.spawn((
                    StreamedSectorEntity {
                        sector_id: sector_id.to_string(),
                    },
                    SpawnedPawn {
                        sector_id: sector_id.to_string(),
                        entry_id: entry.id.clone(),
                        stable_name: stable_name.clone(),
                    },
                    Transform::from_translation(world_pos).with_rotation(base_rotation),
                    Name::new(stable_name),
                ));
                if let Some(profile) = &entry.ai_profile {
                    cmd.insert(CampGuardAi {
                        profile_path: profile.clone(),
                    });
                }
                let id = cmd.id();
                state.spawned_entities.push(id);
                spawned_count += 1;
            }
        }
    }
}

/// Despawns all entities tracked for a sector (markers + spawned pawns).
pub fn deactivate_sector_cleanup(commands: &mut Commands, state: &mut SectorRuntimeState) {
    for entity in state.spawned_entities.drain(..) {
        commands.entity(entity).despawn();
    }
}
