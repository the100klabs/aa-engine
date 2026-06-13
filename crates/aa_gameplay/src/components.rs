use bevy::prelude::*;

/// PlayerState entity — owns ASC split components (AbilityRegistry, AttributeSet).
#[derive(Component, Debug)]
pub struct PlayerState {
    pub player_id: u32,
}

/// Controller that possesses a pawn.
#[derive(Component, Debug)]
pub struct PlayerController {
    pub player_id: u32,
}

/// Link from controller to PlayerState entity.
#[derive(Component, Debug)]
pub struct ControlsPlayer(pub Entity);

/// Marks a possessed pawn body.
#[derive(Component, Debug, Default)]
pub struct Pawn;

/// Active game mode configuration.
#[derive(Resource, Debug, Clone)]
pub struct GameMode {
    pub respawn_delay_secs: f32,
}

impl Default for GameMode {
    fn default() -> Self {
        Self {
            respawn_delay_secs: 3.0,
        }
    }
}
