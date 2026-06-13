use bevy::prelude::*;

/// Kinematic character movement (Rapier integration deferred to Phase 2).
#[derive(Component, Debug, Clone)]
pub struct CharacterMovement {
    pub speed: f32,
    pub wish_dir: Vec3,
    pub velocity: Vec3,
}

impl Default for CharacterMovement {
    fn default() -> Self {
        Self {
            speed: 6.0,
            wish_dir: Vec3::ZERO,
            velocity: Vec3::ZERO,
        }
    }
}
