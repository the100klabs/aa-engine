use bevy::prelude::*;

use crate::manifest::load_asset_manifest;
use crate::registry::AssetRegistry;
use crate::tag_dictionary::{load_tag_dictionary, TagDictionaryResource};

/// Initializes asset resources and loads startup data when present on disk.
pub struct AaAssetsPlugin;

impl Plugin for AaAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetRegistry>()
            .init_resource::<TagDictionaryResource>()
            .add_systems(Startup, (load_tag_dictionary, load_asset_manifest));
    }
}
