use aa_ability::execute_ability_impls;
use aa_core::AaSchedule;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::dash::apply_dash_bursts;
use crate::movement::CharacterMovement;
use crate::projectile::{gather_movement_intent, tick_projectiles};
use crate::rapier::{attach_character_colliders, attach_ground_collider};

pub struct AaPhysicsPlugin;

impl Plugin for AaPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_systems(PostStartup, configure_rapier_gravity)
            .add_systems(
                FixedUpdate,
                (
                    gather_movement_intent.in_set(AaSchedule::MovementIntent),
                    apply_rapier_movement.in_set(AaSchedule::Physics),
                    apply_dash_bursts.in_set(AaSchedule::Physics),
                    tick_projectiles.in_set(AaSchedule::Physics),
                ),
            )
            .add_systems(
                PostUpdate,
                tick_projectiles
                    .after(execute_ability_impls)
                    .in_set(AaSchedule::Physics),
            )
            .add_systems(
                Update,
                (
                    attach_character_movement,
                    attach_character_colliders,
                    attach_ground_collider,
                ),
            );
    }
}

/// Ensures pawns spawned by gameplay receive movement components.
pub fn attach_character_movement(
    mut commands: Commands,
    pawns: Query<Entity, (With<aa_gameplay::Pawn>, Without<CharacterMovement>)>,
) {
    for pawn in &pawns {
        commands.entity(pawn).insert(CharacterMovement::default());
    }
}

/// Drives Rapier kinematic character controllers from movement intent.
pub fn apply_rapier_movement(
    time: Res<Time>,
    mut query: Query<
        (&mut CharacterMovement, &mut KinematicCharacterController),
        With<aa_gameplay::Pawn>,
    >,
) {
    let dt = time.delta_secs();
    for (mut movement, mut controller) in &mut query {
        movement.velocity = movement.wish_dir * movement.speed;
        controller.translation = Some(movement.velocity * dt);
    }
}

fn configure_rapier_gravity(
    mut config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
) {
    for mut rapier_config in &mut config {
        rapier_config.gravity = Vec3::ZERO;
    }
}
