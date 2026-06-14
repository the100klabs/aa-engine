mod assets;
mod cook;
mod inspect;
mod loader;
mod plugin;
mod profile;
mod registry;
pub mod spawn;
mod streaming;

pub use assets::{
    DataLayerState, SectorDescriptorAsset, SectorEntityPlacement, SectorLifecycle,
    SectorRefDescriptor, StreamingPolicy, WorldBounds, WorldDataLayer, WorldDescriptorAsset,
    WorldRegionDescriptor, WorldStreamingBudgets,
};
pub use cook::{cook_world, WorldCookResult};
pub use inspect::{inspect_world, LiveStateJson, WorldInspectResult};
pub use plugin::AaWorldStreamPlugin;
pub use profile::{summarize_trace, ProfileSummaryResult, StreamingProfileTrace};
pub use registry::{SectorDiagnostics, SectorRegistry, SectorRuntimeState};
pub use spawn::{load_spawn_table_from_disk, CampGuardAi, SpawnedPawn};
pub use streaming::{sector_coord_from_position, tick_sector_streaming, StreamingSource, StreamingSourceKind};
