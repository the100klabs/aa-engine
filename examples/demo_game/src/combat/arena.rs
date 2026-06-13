use aa_animation::LocomotionState;
use aa_gameplay::{Pawn, TrainingDummy};
use aa_physics::Projectile;
use aa_scene::Possesses;
use bevy::prelude::*;

/// Updated each frame so ability impls can spawn projectiles at the pawn.
#[derive(Resource, Default)]
pub struct PawnOrigin {
    pub translation: Vec3,
    pub forward: Vec3,
    pub pawn_entity: Option<Entity>,
}

pub fn sync_pawn_origin(
    controllers: Query<&Possesses, With<aa_gameplay::PlayerController>>,
    pawns: Query<&Transform, With<Pawn>>,
    mut origin: ResMut<PawnOrigin>,
) {
    for possesses in &controllers {
        let Ok(transform) = pawns.get(possesses.0) else {
            continue;
        };
        origin.translation = transform.translation;
        origin.forward = transform.forward().as_vec3();
        origin.pawn_entity = Some(possesses.0);
    }
}

pub fn setup_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor_mesh = meshes.add(Plane3d::default().mesh().size(20.0, 20.0));
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let floor_mat = materials.add(Color::srgb(0.2, 0.25, 0.2));
    let player_mat = materials.add(Color::srgb(0.2, 0.5, 0.9));
    let dummy_mat = materials.add(Color::srgb(0.9, 0.3, 0.2));
    let projectile_mat = materials.add(Color::srgb(1.0, 0.6, 0.1));

    commands.insert_resource(VisualHandles {
        cube: cube_mesh,
        player_mat,
        dummy_mat,
        projectile_mat,
    });

    commands.spawn((
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Floor"),
    ));
}

#[derive(Resource)]
pub(crate) struct VisualHandles {
    cube: Handle<Mesh>,
    player_mat: Handle<StandardMaterial>,
    dummy_mat: Handle<StandardMaterial>,
    projectile_mat: Handle<StandardMaterial>,
}

#[allow(clippy::type_complexity)]
pub(crate) fn attach_visuals(
    mut commands: Commands,
    handles: Option<Res<VisualHandles>>,
    pawns: Query<Entity, (With<Pawn>, Without<Mesh3d>)>,
    dummies: Query<Entity, (With<TrainingDummy>, Without<Mesh3d>)>,
    projectiles: Query<(Entity, &Transform), (With<Projectile>, Without<Mesh3d>)>,
) {
    let Some(handles) = handles else {
        return;
    };

    for entity in &pawns {
        commands.entity(entity).insert((
            Mesh3d(handles.cube.clone()),
            MeshMaterial3d(handles.player_mat.clone()),
            LocomotionState::default(),
        ));
    }

    for entity in &dummies {
        commands.entity(entity).insert((
            Mesh3d(handles.cube.clone()),
            MeshMaterial3d(handles.dummy_mat.clone()),
        ));
    }

    for (entity, transform) in &projectiles {
        commands.entity(entity).insert((
            Mesh3d(handles.cube.clone()),
            MeshMaterial3d(handles.projectile_mat.clone()),
        ));
        commands
            .entity(entity)
            .insert(Transform::from_translation(transform.translation).with_scale(Vec3::splat(0.3)));
    }
}
