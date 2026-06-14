use std::path::PathBuf;

use bevy::prelude::*;

use crate::assets::SectorLifecycle;
use crate::profile::StreamingProfileTrace;
use crate::registry::SectorRegistry;
use crate::spawn::{activate_sector_spawns, deactivate_sector_cleanup, is_spawn_table_ref};

/// Identifies why a transform drives sector activation.
#[derive(Debug, Clone)]
pub enum StreamingSourceKind {
    Player { player_id: u32 },
    Camera,
    Script { id: String },
}

/// Runtime streaming source used to compute the active sector window.
#[derive(Component, Debug, Clone)]
pub struct StreamingSource {
    pub id: String,
    pub kind: StreamingSourceKind,
    pub radius_sectors: u32,
    pub priority: u8,
}

/// Active world descriptor path for the current session.
#[derive(Resource, Debug, Clone)]
pub struct ActiveWorld {
    pub asset_path: String,
}

/// Project root used to synchronously load spawn tables during sector activation.
#[derive(Resource, Debug, Clone)]
pub struct StreamingProjectRoot {
    pub path: PathBuf,
}

/// Converts world-space coordinates into sector grid coordinates.
pub fn sector_coord_from_position(position: Vec3, sector_size_m: f32) -> [i32; 2] {
    [
        (position.x / sector_size_m).floor() as i32,
        (position.z / sector_size_m).floor() as i32,
    ]
}

/// Queues sectors inside each source's active window.
pub fn queue_sectors_for_sources(
    registry: &mut SectorRegistry,
    world: &crate::assets::WorldDescriptorAsset,
    sources: &Query<(&Transform, &StreamingSource)>,
) {
    let mut desired = std::collections::HashSet::new();

    for (transform, source) in sources {
        let center = sector_coord_from_position(transform.translation, registry.sector_size_m);
        let radius = source.radius_sectors.max(1) as i32;
        let half_window_x = (registry.active_window[0] as i32).div_euclid(2);
        let half_window_y = (registry.active_window[1] as i32).div_euclid(2);

        for sector in registry.sectors.values() {
            let dx = (sector.ref_descriptor.coord[0] - center[0]).abs();
            let dy = (sector.ref_descriptor.coord[1] - center[1]).abs();
            if dx <= half_window_x.max(radius) && dy <= half_window_y.max(radius) {
                desired.insert(sector.ref_descriptor.id.clone());
            }
        }

        let _ = (&source.kind, &world.id);
    }

    for (sector_id, state) in &mut registry.sectors {
        if desired.contains(sector_id) {
            // Keep Loaded/Loading/Activating/Active sectors on the activation path; only
            // (re-)queue sectors that still need a load pass.
            if matches!(
                state.lifecycle,
                SectorLifecycle::Discovered | SectorLifecycle::Deactivating
            ) {
                state.lifecycle = SectorLifecycle::Queued;
            }
        } else if state.lifecycle == SectorLifecycle::Active {
            state.lifecycle = SectorLifecycle::Deactivating;
        }
    }
}

/// Advances sector lifecycle transitions with per-frame activation budgets.
#[allow(clippy::too_many_arguments)]
pub fn tick_sector_streaming(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<SectorRegistry>,
    world_assets: Res<Assets<crate::assets::WorldDescriptorAsset>>,
    sector_assets: Res<Assets<crate::assets::SectorDescriptorAsset>>,
    sources: Query<(&Transform, &StreamingSource)>,
    project_root: Option<Res<StreamingProjectRoot>>,
    time: Res<Time>,
    mut trace: Option<ResMut<StreamingProfileTrace>>,
) {
    let frame_start = std::time::Instant::now();

    let Some(world) = world_assets.get(&registry.world_handle) else {
        return;
    };

    queue_sectors_for_sources(&mut registry, world, &sources);

    let max_activations = world.streaming.max_activations_per_frame.max(1);
    let max_deactivations = world.streaming.max_deactivations_per_frame.max(1);
    let mut activations = 0u32;
    let mut deactivations = 0u32;

    let mut sector_ids: Vec<String> = registry.sectors.keys().cloned().collect();
    sector_ids.sort_by_key(|id| {
        std::cmp::Reverse(
            registry
                .sectors
                .get(id)
                .map(|s| s.ref_descriptor.priority)
                .unwrap_or(0),
        )
    });
    for sector_id in sector_ids {
        let Some(state) = registry.sectors.get_mut(&sector_id) else {
            continue;
        };

        match state.lifecycle {
            SectorLifecycle::Queued => {
                if let Some(root) = project_root.as_deref()
                    && let Ok(sector) = crate::registry::load_sector_descriptor_from_disk(
                        &root.path,
                        &state.ref_descriptor.path,
                    )
                {
                    state.cached_sector = Some(sector.clone());
                    state.load_started = Some(std::time::Instant::now());
                    state.load_ms = Some(0.0);
                    try_activate_sector(
                        &mut commands,
                        &sector_id,
                        &sector,
                        state,
                        project_root.as_deref(),
                        &mut activations,
                        max_activations,
                        trace.as_mut().map(|t| t.as_mut()),
                    );
                    continue;
                }
                let handle: Handle<crate::assets::SectorDescriptorAsset> =
                    asset_server.load(crate::registry::asset_server_path(&state.ref_descriptor.path));
                state.asset_handle = Some(handle);
                state.load_started = Some(std::time::Instant::now());
                state.lifecycle = SectorLifecycle::Loading;
            }
            SectorLifecycle::Loading => {
                let Some(handle) = &state.asset_handle else {
                    continue;
                };
                if let Some(sector) = sector_assets.get(handle) {
                    state.cached_sector = Some(sector.clone());
                    state.load_ms = state
                        .load_started
                        .map(|started| started.elapsed().as_secs_f32() * 1000.0);
                    if let (Some(load_ms), Some(trace)) = (state.load_ms, trace.as_mut()) {
                        trace.record_load(&sector_id, load_ms, time.elapsed_secs());
                    }
                    try_activate_sector(
                        &mut commands,
                        &sector_id,
                        sector,
                        state,
                        project_root.as_deref(),
                        &mut activations,
                        max_activations,
                        trace.as_mut().map(|t| t.as_mut()),
                    );
                } else if let Some(root) = project_root.as_deref()
                    && let Ok(sector) = crate::registry::load_sector_descriptor_from_disk(
                        &root.path,
                        &state.ref_descriptor.path,
                    )
                {
                    state.cached_sector = Some(sector.clone());
                    state.load_ms = state
                        .load_started
                        .map(|started| started.elapsed().as_secs_f32() * 1000.0);
                    if let (Some(load_ms), Some(trace)) = (state.load_ms, trace.as_mut()) {
                        trace.record_load(&sector_id, load_ms, time.elapsed_secs());
                    }
                    try_activate_sector(
                        &mut commands,
                        &sector_id,
                        &sector,
                        state,
                        project_root.as_deref(),
                        &mut activations,
                        max_activations,
                        trace.as_mut().map(|t| t.as_mut()),
                    );
                }
            }
            SectorLifecycle::Loaded | SectorLifecycle::Activating => {
                if activations < max_activations {
                    if let Some(sector) = state.cached_sector.clone().or_else(|| {
                        state
                            .asset_handle
                            .as_ref()
                            .and_then(|handle| sector_assets.get(handle).cloned())
                    }) {
                        try_activate_sector(
                            &mut commands,
                            &sector_id,
                            &sector,
                            state,
                            project_root.as_deref(),
                            &mut activations,
                            max_activations,
                            trace.as_mut().map(|t| t.as_mut()),
                        );
                    } else if state.lifecycle == SectorLifecycle::Loaded {
                        state.lifecycle = SectorLifecycle::Activating;
                    }
                }
            }
            SectorLifecycle::Deactivating => {
                if deactivations < max_deactivations {
                    deactivate_sector_cleanup(&mut commands, state);
                    state.lifecycle = SectorLifecycle::Loaded;
                    deactivations += 1;
                    if let Some(trace) = trace.as_mut() {
                        trace.deactivation_count += 1;
                    }
                }
            }
            SectorLifecycle::Discovered | SectorLifecycle::Active | SectorLifecycle::Unloading => {}
        }
    }

    if let Some(trace) = trace.as_mut() {
        let active = registry
            .sectors
            .values()
            .filter(|s| s.lifecycle == SectorLifecycle::Active)
            .count() as u32;
        trace.max_active_sectors = trace.max_active_sectors.max(active);
        trace.elapsed_secs = time.elapsed_secs();
        let had_sector_crossing = activations > 0 || deactivations > 0;
        // Ignore startup/render-init frames; only count hitches on sector crossing frames.
        if time.elapsed_secs() >= 2.0 {
            trace.record_frame(
                frame_start.elapsed().as_secs_f32() * 1000.0,
                had_sector_crossing,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn try_activate_sector(
    commands: &mut Commands,
    sector_id: &str,
    sector: &crate::assets::SectorDescriptorAsset,
    state: &mut crate::registry::SectorRuntimeState,
    project_root: Option<&StreamingProjectRoot>,
    activations: &mut u32,
    max_activations: u32,
    mut trace: Option<&mut StreamingProfileTrace>,
) {
    if *activations >= max_activations {
        state.lifecycle = SectorLifecycle::Loaded;
        return;
    }
    state.lifecycle = SectorLifecycle::Activating;
    spawn_sector_entities(commands, sector_id, sector, &mut state.spawned_entities);
    if let Some(root) = project_root {
        activate_sector_spawns(
            commands,
            &root.path,
            sector_id,
            &sector.entities,
            state,
        );
    }
    state.lifecycle = SectorLifecycle::Active;
    *activations += 1;
    if let Some(trace) = trace.as_mut() {
        trace.activation_count += 1;
    }
}

fn spawn_sector_entities(
    commands: &mut Commands,
    sector_id: &str,
    sector: &crate::assets::SectorDescriptorAsset,
    spawned_entities: &mut Vec<Entity>,
) {
    for placement in &sector.entities {
        if is_spawn_table_ref(&placement.prefab) {
            continue;
        }
        let entity = commands
            .spawn((
                crate::registry::StreamedSectorEntity {
                    sector_id: sector_id.to_string(),
                },
                placement_to_transform(placement),
                Name::new(format!("{sector_id}:{}", placement.prefab)),
            ))
            .id();
        spawned_entities.push(entity);
    }
}

fn placement_to_transform(placement: &crate::assets::SectorEntityPlacement) -> Transform {
    let rotation = Quat::from_rotation_y(placement.transform.rotation_y_degrees.to_radians());
    Transform {
        translation: Vec3::from_array(placement.transform.translation),
        rotation,
        scale: Vec3::from_array(placement.transform.scale),
    }
}
