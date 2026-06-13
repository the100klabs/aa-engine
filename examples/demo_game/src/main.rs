//! Phase 1 combat slice — move, aim, fire, damage, death, respawn.

mod camera;
mod combat;
mod hud;
mod playtest;

use aa_ability::AaAbilityPlugin;
use aa_animation::AaAnimationPlugin;
use aa_assets::AaAssetsPlugin;
use aa_core::{init_project, AaCorePlugin, AaSchedule};
use aa_experience::AaExperiencePlugin;
use aa_gameplay::AaGameplayPlugin;
use aa_input::AaInputPlugin;
use aa_physics::AaPhysicsPlugin;
use aa_scene::AaScenePlugin;
use aa_tags::AaTagsPlugin;
use bevy::prelude::*;

use camera::{
    apply_camera_relative_movement, apply_pawn_facing, camera_follow, setup_combat_camera,
    update_aim, AimState,
};
use combat::{
    attach_visuals, register_ability_impls, route_ability_input, setup_arena, sync_pawn_origin,
    PawnOrigin,
};
use hud::{setup_hud, update_hud};

fn main() {
    if std::env::var("AA_PLAYTEST").as_deref() == Ok("1") {
        playtest::run();
        return;
    }

    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    App::new()
        .add_plugins(DefaultPlugins.set(init_project(&project_root)))
        .add_plugins(AaCorePlugin::default())
        .add_plugins(AaAssetsPlugin)
        .add_plugins(AaTagsPlugin)
        .add_plugins(AaInputPlugin)
        .add_plugins(AaAbilityPlugin)
        .add_plugins(AaExperiencePlugin {
            default_experience: "experiences/demo".into(),
        })
        .add_plugins(AaScenePlugin)
        .add_plugins(AaGameplayPlugin)
        .add_plugins(AaPhysicsPlugin)
        .add_plugins(AaAnimationPlugin)
        .init_resource::<PawnOrigin>()
        .init_resource::<AimState>()
        .add_systems(
            Startup,
            (
                register_ability_impls,
                setup_arena,
                setup_combat_camera,
                setup_hud,
            ),
        )
        .add_systems(
            PreUpdate,
            route_ability_input.in_set(AaSchedule::AbilityInput),
        )
        .add_systems(
            Update,
            (
                update_aim,
                apply_pawn_facing,
                sync_pawn_origin,
                camera_follow,
                attach_visuals,
                update_hud,
            ),
        )
        .add_systems(
            FixedUpdate,
            apply_camera_relative_movement.in_set(AaSchedule::MovementIntent),
        )
        .run();
}
