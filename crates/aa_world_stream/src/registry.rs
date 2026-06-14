use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bevy::prelude::*;
use serde::Serialize;

use crate::assets::{
    DataLayerState, SectorDescriptorAsset, SectorLifecycle, SectorRefDescriptor, WorldDescriptorAsset,
};

/// Marks entities spawned by a streamed sector so unload can clean them up.
#[derive(Component, Debug, Clone)]
pub struct StreamedSectorEntity {
    #[allow(dead_code)]
    pub sector_id: String,
}

/// Runtime state tracked per authored sector.
#[derive(Debug, Clone)]
pub struct SectorRuntimeState {
    pub ref_descriptor: SectorRefDescriptor,
    pub lifecycle: SectorLifecycle,
    pub asset_handle: Option<Handle<SectorDescriptorAsset>>,
    pub cached_sector: Option<SectorDescriptorAsset>,
    pub spawned_entities: Vec<Entity>,
    pub load_started: Option<Instant>,
    pub load_ms: Option<f32>,
    pub layer_states: HashMap<String, DataLayerState>,
}

impl SectorRuntimeState {
    pub fn new(descriptor: SectorRefDescriptor, default_layers: &[(String, DataLayerState)]) -> Self {
        let mut layer_states = HashMap::new();
        for (layer, state) in default_layers {
            layer_states.insert(layer.clone(), *state);
        }
        Self {
            ref_descriptor: descriptor,
            lifecycle: SectorLifecycle::Discovered,
            asset_handle: None,
            cached_sector: None,
            spawned_entities: Vec::new(),
            load_started: None,
            load_ms: None,
            layer_states,
        }
    }

    pub fn required_layers_active(&self) -> bool {
        self.ref_descriptor.required_layers.iter().all(|layer| {
            matches!(
                self.layer_states.get(layer),
                Some(DataLayerState::Active) | Some(DataLayerState::Loaded)
            )
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SectorDiagnostics {
    pub id: String,
    pub loaded: bool,
    pub active: bool,
    pub entity_count: u32,
    pub memory_estimate_mb: f32,
    pub load_ms: Option<f32>,
    pub refs: Vec<String>,
    pub layers: Vec<String>,
}

/// Registry of all sectors declared by the active world descriptor.
#[derive(Resource, Debug)]
pub struct SectorRegistry {
    pub world_id: String,
    pub world_asset_path: String,
    pub world_handle: Handle<WorldDescriptorAsset>,
    pub sector_size_m: f32,
    pub active_window: [u32; 2],
    pub sectors: HashMap<String, SectorRuntimeState>,
}

impl SectorRegistry {
    pub fn from_world(
        world_asset_path: impl Into<String>,
        world_handle: Handle<WorldDescriptorAsset>,
        world: &WorldDescriptorAsset,
    ) -> Self {
        let default_layers: Vec<(String, DataLayerState)> = world
            .data_layers
            .iter()
            .map(|layer| (layer.id.clone(), layer.default_state))
            .collect();

        let mut sectors = HashMap::new();
        for region in &world.regions {
            for sector_ref in &region.sectors {
                sectors.insert(
                    sector_ref.id.clone(),
                    SectorRuntimeState::new(sector_ref.clone(), &default_layers),
                );
            }
        }

        Self {
            world_id: world.id.clone(),
            world_asset_path: world_asset_path.into(),
            world_handle,
            sector_size_m: world.sector_size_m,
            active_window: world.active_window,
            sectors,
        }
    }

    pub fn diagnostics(&self, sector_assets: &Assets<SectorDescriptorAsset>) -> Vec<SectorDiagnostics> {
        self.sectors
            .values()
            .map(|state| {
                let refs = state
                    .asset_handle
                    .as_ref()
                    .and_then(|handle| sector_assets.get(handle))
                    .map(collect_sector_refs)
                    .unwrap_or_default();

                SectorDiagnostics {
                    id: state.ref_descriptor.id.clone(),
                    loaded: matches!(
                        state.lifecycle,
                        SectorLifecycle::Loaded
                            | SectorLifecycle::Activating
                            | SectorLifecycle::Active
                            | SectorLifecycle::Deactivating
                    ),
                    active: state.lifecycle == SectorLifecycle::Active,
                    entity_count: state.spawned_entities.len() as u32,
                    memory_estimate_mb: state.spawned_entities.len() as f32 * 0.01,
                    load_ms: state.load_ms,
                    refs,
                    layers: state.ref_descriptor.required_layers.clone(),
                }
            })
            .collect()
    }
}

fn collect_sector_refs(sector: &SectorDescriptorAsset) -> Vec<String> {
    let mut refs: Vec<String> = sector
        .entities
        .iter()
        .map(|entity| entity.prefab.clone())
        .collect();
    if let Some(nav) = &sector.navmesh {
        refs.push(nav.clone());
    }
    if let Some(hlod) = &sector.hlod {
        refs.push(hlod.clone());
    }
    refs.sort();
    refs.dedup();
    refs
}

/// Loads a world descriptor from disk without spinning up Bevy (used by CLI inspect).
pub fn load_world_descriptor_from_disk(project_root: &Path, world_asset: &str) -> Result<WorldDescriptorAsset, String> {
    let path = resolve_asset_path(project_root, world_asset);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    if let Ok(data) = ron::from_str::<crate::assets::WorldDescriptorAssetData>(&text) {
        return Ok(WorldDescriptorAsset::from(data));
    }
    let data: crate::assets::WorldDescriptorAssetData = ron::from_str(&text)
        .map_err(|e| format!("invalid world RON {}: {e}", path.display()))?;
    Ok(WorldDescriptorAsset::from(data))
}

/// Loads a sector descriptor from disk without spinning up Bevy.
pub fn load_sector_descriptor_from_disk(project_root: &Path, sector_asset: &str) -> Result<SectorDescriptorAsset, String> {
    let path = resolve_asset_path(project_root, sector_asset);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    if let Ok(data) = ron::from_str::<crate::assets::SectorDescriptorAssetData>(&text) {
        return Ok(SectorDescriptorAsset::from(data));
    }
    let data: crate::assets::SectorDescriptorAssetData = ron::from_str(&text)
        .map_err(|e| format!("invalid sector RON {}: {e}", path.display()))?;
    Ok(SectorDescriptorAsset::from(data))
}

pub fn resolve_world_asset_path(project_root: &Path, world: &str) -> PathBuf {
    if world.ends_with(".ron") {
        resolve_asset_path(project_root, world)
    } else {
        resolve_asset_path(project_root, &format!("worlds/{world}.ron"))
    }
}

fn resolve_asset_path(project_root: &Path, asset: &str) -> PathBuf {
    resolve_asset_path_for_disk(project_root, asset)
}

pub fn resolve_asset_path_for_disk(project_root: &Path, asset: &str) -> PathBuf {
    let candidate = project_root.join(asset);
    if candidate.is_file() {
        return candidate;
    }
    if !asset.starts_with("assets/") {
        let with_assets = project_root.join("assets").join(asset);
        if with_assets.is_file() {
            return with_assets;
        }
    }
    candidate
}

pub fn asset_server_path(sector_asset: &str) -> String {
    sector_asset
        .strip_prefix("assets/")
        .unwrap_or(sector_asset)
        .to_string()
}

pub fn project_relative_path(project_root: &Path, absolute: &Path) -> String {
    absolute
        .strip_prefix(project_root)
        .unwrap_or(absolute)
        .to_string_lossy()
        .replace('\\', "/")
}
