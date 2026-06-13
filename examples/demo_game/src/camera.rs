use aa_input::{axis2d, InputActionEvent};
use aa_scene::Possesses;
use aa_physics::CharacterMovement;
use aa_gameplay::{Pawn, PlayerController};
use bevy::prelude::*;

/// Horizontal aim angle (radians) driven by mouse look.
#[derive(Resource)]
pub struct AimState {
    pub yaw: f32,
}

impl Default for AimState {
    fn default() -> Self {
        Self { yaw: 0.0 }
    }
}

/// Marks the combat follow camera.
#[derive(Component)]
pub struct CombatCamera;

pub fn setup_combat_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        CombatCamera,
        Transform::from_xyz(0.0, 6.0, 10.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
}

pub fn update_aim(
    mut input_events: MessageReader<InputActionEvent>,
    mut aim: ResMut<AimState>,
) {
    for event in input_events.read() {
        if event.action.0 != "Look" {
            continue;
        }
        let delta = axis2d(event.value);
        aim.yaw -= delta.x * 0.004;
    }
}

pub fn apply_pawn_facing(
    aim: Res<AimState>,
    controllers: Query<&Possesses, With<PlayerController>>,
    mut pawns: Query<&mut Transform, With<Pawn>>,
) {
    let rotation = Quat::from_rotation_y(aim.yaw);
    for possesses in &controllers {
        let Ok(mut transform) = pawns.get_mut(possesses.0) else {
            continue;
        };
        transform.rotation = rotation;
    }
}

pub fn apply_camera_relative_movement(
    aim: Res<AimState>,
    controllers: Query<&Possesses, With<PlayerController>>,
    mut movement: Query<&mut CharacterMovement>,
) {
    let rotation = Quat::from_rotation_y(aim.yaw);
    for possesses in &controllers {
        let Ok(mut character) = movement.get_mut(possesses.0) else {
            continue;
        };
        if character.wish_dir != Vec3::ZERO {
            character.wish_dir = (rotation * character.wish_dir).normalize_or_zero();
        }
    }
}

pub fn camera_follow(
    aim: Res<AimState>,
    controllers: Query<&Possesses, With<PlayerController>>,
    pawns: Query<&Transform, With<Pawn>>,
    mut cameras: Query<&mut Transform, With<CombatCamera>>,
) {
    let Some(possesses) = controllers.iter().next() else {
        return;
    };
    let Ok(pawn_transform) = pawns.get(possesses.0) else {
        return;
    };

    let offset = Vec3::new(0.0, 5.0, 10.0);
    let rotated_offset = Quat::from_rotation_y(aim.yaw) * offset;

    for mut camera in &mut cameras {
        camera.translation = pawn_transform.translation + rotated_offset;
        camera.look_at(pawn_transform.translation + Vec3::Y, Vec3::Y);
    }
}
