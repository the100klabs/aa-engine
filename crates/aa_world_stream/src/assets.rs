use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Axis-aligned bounds in meters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataLayerState {
    Active,
    #[default]
    Loaded,
    Unloaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDataLayer {
    pub id: String,
    pub default_state: DataLayerState,
    #[serde(default)]
    pub server_authoritative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamingPolicy {
    #[serde(default = "default_max_activations")]
    pub max_activations_per_frame: u32,
    #[serde(default = "default_max_deactivations")]
    pub max_deactivations_per_frame: u32,
    #[serde(default = "default_load_budget")]
    pub load_latency_budget_ms: f32,
    #[serde(default = "default_hitch_budget")]
    pub crossing_hitch_budget_ms: f32,
    #[serde(default)]
    pub multi_source: bool,
}

fn default_max_activations() -> u32 {
    2
}

fn default_max_deactivations() -> u32 {
    2
}

fn default_load_budget() -> f32 {
    500.0
}

fn default_hitch_budget() -> f32 {
    6.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldStreamingBudgets {
    #[serde(default)]
    pub authored_objects: u32,
    #[serde(default)]
    pub visible_instanced_props: u32,
    #[serde(default)]
    pub full_ai_agents: u32,
    #[serde(default)]
    pub low_lod_agents: u32,
    #[serde(default)]
    pub memory_mb: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorRefDescriptor {
    pub id: String,
    pub coord: [i32; 2],
    pub path: String,
    #[serde(default)]
    pub required_layers: Vec<String>,
    #[serde(default = "default_priority")]
    pub priority: u8,
}

fn default_priority() -> u8 {
    128
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldRegionDescriptor {
    pub id: String,
    pub coord: [i32; 2],
    pub bounds_m: WorldBounds,
    pub sectors: Vec<SectorRefDescriptor>,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct WorldDescriptorAsset {
    pub schema_version: u32,
    pub id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub bounds_m: WorldBounds,
    pub sector_size_m: f32,
    pub active_window: [u32; 2],
    pub streaming: StreamingPolicy,
    pub data_layers: Vec<WorldDataLayer>,
    pub regions: Vec<WorldRegionDescriptor>,
    pub budgets: WorldStreamingBudgets,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "WorldDescriptor")]
pub(crate) struct WorldDescriptorAssetData {
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub bounds_m: WorldBounds,
    pub sector_size_m: f32,
    pub active_window: ActiveWindow,
    #[serde(default)]
    pub streaming: StreamingPolicy,
    pub data_layers: Vec<WorldDataLayer>,
    pub regions: Vec<WorldRegionDescriptor>,
    #[serde(default)]
    pub budgets: WorldStreamingBudgets,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ActiveWindow {
    pub x: u32,
    pub y: u32,
}

impl From<WorldDescriptorAssetData> for WorldDescriptorAsset {
    fn from(data: WorldDescriptorAssetData) -> Self {
        Self {
            schema_version: data.schema_version,
            id: data.id,
            display_name: if data.display_name.is_empty() {
                None
            } else {
                Some(data.display_name)
            },
            description: if data.description.is_empty() {
                None
            } else {
                Some(data.description)
            },
            bounds_m: data.bounds_m,
            sector_size_m: data.sector_size_m,
            active_window: [data.active_window.x, data.active_window.y],
            streaming: data.streaming,
            data_layers: data.data_layers,
            regions: data.regions,
            budgets: data.budgets,
        }
    }
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct SectorDescriptorAsset {
    pub schema_version: u32,
    pub id: String,
    pub coord: [i32; 2],
    pub bounds: WorldBounds,
    pub data_layers: Vec<String>,
    pub entities: Vec<SectorEntityPlacement>,
    pub navmesh: Option<String>,
    pub hlod: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorEntityPlacement {
    pub prefab: String,
    pub transform: SectorTransform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorTransform {
    pub translation: [f32; 3],
    #[serde(default)]
    pub rotation_y_degrees: f32,
    #[serde(default = "default_scale")]
    pub scale: [f32; 3],
}

fn default_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

#[derive(Debug, Deserialize)]
#[serde(rename = "SectorDescriptor")]
pub(crate) struct SectorDescriptorAssetData {
    pub schema_version: u32,
    pub id: String,
    pub coord: [i32; 2],
    pub bounds: WorldBounds,
    pub data_layers: Vec<String>,
    #[serde(default)]
    pub entities: Vec<SectorEntityPlacement>,
    pub navmesh: Option<String>,
    pub hlod: Option<String>,
}

impl From<SectorDescriptorAssetData> for SectorDescriptorAsset {
    fn from(data: SectorDescriptorAssetData) -> Self {
        Self {
            schema_version: data.schema_version,
            id: data.id,
            coord: data.coord,
            bounds: data.bounds,
            data_layers: data.data_layers,
            entities: data.entities,
            navmesh: data.navmesh,
            hlod: data.hlod,
        }
    }
}

/// Sector lifecycle states used by the streaming scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectorLifecycle {
    Discovered,
    Queued,
    Loading,
    Loaded,
    Activating,
    Active,
    Deactivating,
    Unloading,
}
