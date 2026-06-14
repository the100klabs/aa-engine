//! Open World Studio — AAA-track streamed world prototype.
//!
//! Boots a partitioned world descriptor, activates sectors around the player camera,
//! and exposes runtime state for `aa world inspect --live`.

mod playtest;

use aa_core::{init_project, AaCorePlugin};
use aa_world_stream::{AaWorldStreamPlugin, StreamingSource, StreamingSourceKind};
use bevy::prelude::*;

fn main() {
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    if std::env::var("AA_PLAYTEST").as_deref() == Ok("1") {
        playtest::run();
        return;
    }

    if std::env::args().any(|arg| arg == "--describe") {
        println!("open_world_studio runtime: streamed world prototype with sector activation");
        return;
    }

    App::new()
        .add_plugins(DefaultPlugins.set(init_project(&project_root)))
        .add_plugins(AaCorePlugin::default())
        .add_plugins(AaWorldStreamPlugin {
            world_asset: "worlds/open_world_studio.ron".into(),
            project_root: project_root.clone(),
        })
        .add_systems(Startup, (setup_scene, spawn_streaming_source))
        .add_systems(Update, follow_camera)
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(32.0, 18.0, 32.0).looking_at(Vec3::new(28.0, 0.0, -20.0), Vec3::Y),
        Name::new("OpenWorldCamera"),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.6, 0.0)),
    ));
}

fn spawn_streaming_source(mut commands: Commands) {
    commands.spawn((
        StreamingSource {
            id: "player_0".into(),
            kind: StreamingSourceKind::Player { player_id: 0 },
            radius_sectors: 1,
            priority: 255,
        },
        Transform::from_xyz(32.0, 0.0, 32.0),
        Name::new("StreamingSource"),
    ));
}

fn follow_camera(
    source: Query<&Transform, With<StreamingSource>>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<StreamingSource>)>,
) {
    let Ok(source_transform) = source.single() else {
        return;
    };
    for mut camera in &mut cameras {
        camera.translation = source_transform.translation + Vec3::new(0.0, 18.0, 24.0);
        camera.look_at(source_transform.translation, Vec3::Y);
    }
}
