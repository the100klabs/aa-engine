use bevy::{asset::AssetPlugin, prelude::*};
use std::path::Path;

use crate::paths::set_project_root;
use crate::AaCorePlugin;

/// Initializes the AA project root and returns a Bevy [`AssetPlugin`] for its `assets/` folder.
pub fn init_project(root: impl AsRef<Path>) -> AssetPlugin {
    let root = root.as_ref().to_path_buf();
    set_project_root(&root);
    AssetPlugin {
        file_path: root.join("assets").to_string_lossy().into_owned(),
        ..default()
    }
}

/// Registers the core AA plugin. Domain plugins (`aa_assets`, `aa_scene`, …) follow separately.
pub fn add_core_plugin(app: &mut App) {
    app.add_plugins(AaCorePlugin::default());
}
