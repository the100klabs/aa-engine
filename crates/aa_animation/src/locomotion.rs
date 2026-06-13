use bevy::prelude::*;

/// Minimal locomotion FSM state (full anim graph in Phase 2).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocomotionState {
    #[default]
    Idle,
    Walk,
    Run,
}
