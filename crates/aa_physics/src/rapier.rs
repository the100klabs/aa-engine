use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Marker: entity participates in Rapier collision.
#[derive(Component, Debug)]
pub struct PhysicsCharacter;

/// Static environment collider marker.
#[derive(Component, Debug)]
pub struct PhysicsGround;

/// Rapier bundle for kinematic player/NPC bodies.
pub fn character_collider_bundle() -> impl Bundle {
    (
        PhysicsCharacter,
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(0.5, 0.25),
        KinematicCharacterController {
            up: Vec3::Y,
            offset: CharacterLength::Absolute(0.05),
            max_slope_climb_angle: 45.0_f32.to_radians(),
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
    )
}

/// Rapier bundle for static floor tiles.
pub fn ground_collider_bundle() -> impl Bundle {
    (
        PhysicsGround,
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.1, 10.0),
    )
}

/// Attaches Rapier colliders to pawns and dummies once spawned.
pub fn attach_character_colliders(
    mut commands: Commands,
    pawns: Query<Entity, (With<aa_gameplay::Pawn>, Without<PhysicsCharacter>)>,
    dummies: Query<Entity, (With<aa_gameplay::TrainingDummy>, Without<PhysicsCharacter>)>,
) {
    for entity in &pawns {
        commands.entity(entity).insert(character_collider_bundle());
    }
    for entity in &dummies {
        commands.entity(entity).insert(character_collider_bundle());
    }
}

/// Attaches a static collider to the arena floor entity.
pub fn attach_ground_collider(
    mut commands: Commands,
    floors: Query<(Entity, &Name), Without<PhysicsGround>>,
) {
    for (entity, name) in &floors {
        if name.as_str() == "Floor" {
            commands.entity(entity).insert(ground_collider_bundle());
        }
    }
}
