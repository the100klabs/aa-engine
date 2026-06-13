use bevy::prelude::*;

use crate::loader::{PrefabAssetLoader, SceneAssetLoader};
use crate::prefab::PrefabAsset;
use crate::scene::SceneAsset;

/// Registers scene/prefab asset loaders (no default camera — combat demo owns the camera).
pub struct AaScenePlugin;

impl Plugin for AaScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<PrefabAsset>()
            .init_asset::<SceneAsset>()
            .init_asset_loader::<PrefabAssetLoader>()
            .init_asset_loader::<SceneAssetLoader>();
    }
}
