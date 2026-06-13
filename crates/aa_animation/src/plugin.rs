use aa_core::AaSchedule;
use aa_physics::CharacterMovement;
use bevy::prelude::*;

use crate::locomotion::LocomotionState;

pub struct AaAnimationPlugin;

impl Plugin for AaAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            sync_locomotion_state
                .in_set(AaSchedule::Animation)
                .after(aa_core::AaSchedule::InitState),
        );
    }
}

fn sync_locomotion_state(
    mut query: Query<(&CharacterMovement, &mut LocomotionState)>,
) {
    for (movement, mut state) in &mut query {
        let speed = movement.velocity.length();
        *state = if speed < 0.1 {
            LocomotionState::Idle
        } else if speed < 4.0 {
            LocomotionState::Walk
        } else {
            LocomotionState::Run
        };
    }
}
