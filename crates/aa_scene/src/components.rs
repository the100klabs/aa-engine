use bevy::prelude::*;

/// Marks entities spawned from prefabs/scenes until init systems finish setup.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct PendingInit;
