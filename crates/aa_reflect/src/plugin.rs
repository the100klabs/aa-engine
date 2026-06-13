use bevy::prelude::*;

use crate::ReflectRegistry;

/// Initializes reflection resources used across the AA Engine stack.
pub struct AaReflectPlugin;

impl Plugin for AaReflectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReflectRegistry>();
    }
}
