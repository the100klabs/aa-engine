use std::path::Path;

use aa_core::{init_project, AaCorePlugin};
use bevy::prelude::*;

use crate::assets::SectorLifecycle;
use crate::inspect::{inspect_world, LiveStateJson, WorldInspectResult};
use crate::plugin::AaWorldStreamPlugin;
use crate::registry::{
    load_world_descriptor_from_disk, project_relative_path, resolve_world_asset_path,
    SectorRegistry,
};
use crate::streaming::{StreamingSource, StreamingSourceKind};

/// Runs a minimal headless streaming session and merges registry state into inspect output.
pub fn inspect_world_live(project_root: &Path, world: &str) -> WorldInspectResult {
    let world_asset = world_asset_server_path(project_root, world);
    let live_state = snapshot_live_registry(project_root, &world_asset);
    inspect_world(project_root, world, Some(live_state))
}

fn world_asset_server_path(project_root: &Path, world: &str) -> String {
    let path = resolve_world_asset_path(project_root, world);
    let relative = project_relative_path(project_root, &path);
    relative
        .strip_prefix("assets/")
        .unwrap_or(&relative)
        .to_string()
}

fn snapshot_live_registry(project_root: &Path, world_asset: &str) -> LiveStateJson {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, init_project(project_root)))
        .add_plugins(AaCorePlugin::default())
        .add_plugins(AaWorldStreamPlugin {
            world_asset: world_asset.to_string(),
            project_root: project_root.to_path_buf(),
        })
        .add_systems(Startup, spawn_inspect_streaming_source);

    if let Ok(world_data) = load_world_descriptor_from_disk(project_root, world_asset) {
        let handle = app
            .world_mut()
            .resource_mut::<Assets<crate::assets::WorldDescriptorAsset>>()
            .add(world_data);
        let world = app
            .world()
            .resource::<Assets<crate::assets::WorldDescriptorAsset>>()
            .get(&handle)
            .expect("sync-loaded world must be present")
            .clone();
        app.world_mut()
            .insert_resource(SectorRegistry::from_world(world_asset, handle, &world));
    }

    const MIN_FRAMES: u32 = 10;
    const MAX_FRAMES: u32 = 300;
    for frame in 0..MAX_FRAMES {
        app.update();
        if frame >= MIN_FRAMES && registry_has_active_sectors(&app) {
            break;
        }
    }

    let streaming_sources = collect_streaming_source_ids(app.world_mut());
    if let Some(registry) = app.world().get_resource::<SectorRegistry>() {
        live_state_from_registry(registry, streaming_sources)
    } else {
        LiveStateJson {
            connected: false,
            active_sectors: Vec::new(),
            loaded_sectors: Vec::new(),
            streaming_sources,
            pending_loads: Vec::new(),
        }
    }
}

fn registry_has_active_sectors(app: &App) -> bool {
    app.world()
        .get_resource::<SectorRegistry>()
        .is_some_and(|registry| {
            registry
                .sectors
                .values()
                .any(|state| state.lifecycle == SectorLifecycle::Active)
        })
}

fn spawn_inspect_streaming_source(mut commands: Commands) {
    commands.spawn((
        StreamingSource {
            id: "player_0".into(),
            kind: StreamingSourceKind::Player { player_id: 0 },
            radius_sectors: 1,
            priority: 255,
        },
        Transform::from_xyz(32.0, 0.0, 32.0),
        Name::new("InspectStreamingSource"),
    ));
}

fn collect_streaming_source_ids(world: &mut World) -> Vec<String> {
    let mut sources = Vec::new();
    let mut query = world.query::<&StreamingSource>();
    for source in query.iter(world) {
        sources.push(source.id.clone());
    }
    sources.sort();
    sources.dedup();
    sources
}

fn live_state_from_registry(registry: &SectorRegistry, streaming_sources: Vec<String>) -> LiveStateJson {
    let mut active_sectors = Vec::new();
    let mut loaded_sectors = Vec::new();
    let mut pending_loads = Vec::new();

    for (sector_id, state) in &registry.sectors {
        match state.lifecycle {
            SectorLifecycle::Active => {
                active_sectors.push(sector_id.clone());
                loaded_sectors.push(sector_id.clone());
            }
            SectorLifecycle::Loaded
            | SectorLifecycle::Activating
            | SectorLifecycle::Deactivating => {
                loaded_sectors.push(sector_id.clone());
            }
            SectorLifecycle::Queued | SectorLifecycle::Loading => {
                pending_loads.push(sector_id.clone());
            }
            SectorLifecycle::Discovered | SectorLifecycle::Unloading => {}
        }
    }

    active_sectors.sort();
    loaded_sectors.sort();
    pending_loads.sort();

    LiveStateJson {
        connected: true,
        active_sectors,
        loaded_sectors,
        streaming_sources,
        pending_loads,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn inspect_open_world_studio_live_reports_registry_sectors() {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/open_world_studio");
        let result = inspect_world_live(&project_root, "open_world_studio");
        assert_eq!(result.world, "open_world_studio");
        assert!(result.sector_count >= 1024, "expected 32x32 sector grid (64 km²)");
        let live = result.live.expect("live inspect must include registry snapshot");
        assert!(live.connected);
        assert!(live.streaming_sources.contains(&"player_0".to_string()));
        assert!(!live.active_sectors.is_empty(), "expected active sectors around player source");
        assert!(
            live.active_sectors.iter().any(|id| id.contains("_0_")),
            "expected sectors near origin to be active: {:?}",
            live.active_sectors
        );
    }
}
