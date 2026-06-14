use bevy::prelude::*;

use crate::assets::{SectorDescriptorAsset, WorldDescriptorAsset};
use crate::loader::{SectorDescriptorAssetLoader, WorldDescriptorAssetLoader};
use crate::registry::SectorRegistry;
use crate::profile::StreamingProfileTrace;
use crate::streaming::{tick_sector_streaming, StreamingProjectRoot};

/// Bevy plugin that loads world/sector descriptors and streams sector entities.
pub struct AaWorldStreamPlugin {
    pub world_asset: String,
    pub project_root: std::path::PathBuf,
}

#[derive(Resource, Debug)]
struct PendingWorldRegistry {
    asset_path: String,
    handle: Handle<WorldDescriptorAsset>,
}

impl Plugin for AaWorldStreamPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<WorldDescriptorAsset>()
            .init_asset::<SectorDescriptorAsset>()
            .init_asset_loader::<WorldDescriptorAssetLoader>()
            .init_asset_loader::<SectorDescriptorAssetLoader>()
            .insert_resource(crate::streaming::ActiveWorld {
                asset_path: self.world_asset.clone(),
            })
            .insert_resource(StreamingProjectRoot {
                path: self.project_root.clone(),
            })
            .init_resource::<StreamingProfileTrace>()
            .add_systems(Startup, bootstrap_world_registry)
            .add_systems(
                Update,
                (
                    finish_pending_world_registry,
                    tick_sector_streaming.run_if(resource_exists::<SectorRegistry>),
                )
                    .chain(),
            );
    }
}

fn bootstrap_world_registry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_world: Res<crate::streaming::ActiveWorld>,
    world_assets: Res<Assets<WorldDescriptorAsset>>,
) {
    let handle: Handle<WorldDescriptorAsset> = asset_server.load(active_world.asset_path.clone());
    if let Some(world) = world_assets.get(&handle) {
        commands.insert_resource(SectorRegistry::from_world(
            active_world.asset_path.clone(),
            handle.clone(),
            world,
        ));
    } else {
        commands.insert_resource(PendingWorldRegistry {
            asset_path: active_world.asset_path.clone(),
            handle,
        });
    }
}

fn finish_pending_world_registry(
    mut commands: Commands,
    pending: Option<Res<PendingWorldRegistry>>,
    world_assets: Res<Assets<WorldDescriptorAsset>>,
) {
    let Some(pending) = pending else {
        return;
    };
    let Some(world) = world_assets.get(&pending.handle) else {
        return;
    };
    commands.insert_resource(SectorRegistry::from_world(
        pending.asset_path.clone(),
        pending.handle.clone(),
        world,
    ));
    commands.remove_resource::<PendingWorldRegistry>();
}
