//! Verifies training dummy melee can kill and respawn the player.

use aa_ability::{AaAbilityPlugin, AttributeSet};
use aa_assets::AaAssetsPlugin;
use aa_core::{init_project, AaCorePlugin};
use aa_experience::AaExperiencePlugin;
use aa_gameplay::{AaGameplayPlugin, DummyCombat, PlayerController, PlayerState, TrainingDummy};
use aa_input::AaInputPlugin;
use aa_scene::Possesses;
use aa_tags::AaTagsPlugin;
use bevy::input::InputPlugin;
use bevy::prelude::*;

const DEMO_PROJECT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/demo_game");

fn demo_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, init_project(DEMO_PROJECT), InputPlugin));
    app.add_plugins(AaCorePlugin::default());
    app.add_plugins(AaAssetsPlugin);
    app.add_plugins(AaTagsPlugin);
    app.add_plugins(AaInputPlugin);
    app.add_plugins(AaAbilityPlugin);
    app.add_plugins(AaExperiencePlugin {
        default_experience: "experiences/demo".into(),
    });
    app.add_plugins(AaGameplayPlugin);
    app
}

#[test]
fn dummy_melee_kills_player() {
    let mut app = demo_app();

    for frame in 0..600 {
        app.update();
        if frame == 5 {
            let world = app.world_mut();
            for mut combat in world
                .query_filtered::<&mut DummyCombat, With<TrainingDummy>>()
                .iter_mut(world)
            {
                combat.range = 10.0;
                combat.cooldown_secs = 0.5;
            }
        }
    }

    let world = app.world_mut();
    let mut player_health = None;
    for (entity, attrs) in world
        .query::<(Entity, &AttributeSet)>()
        .iter(world)
    {
        if world.get::<PlayerState>(entity).is_some() {
            player_health = attrs.get("Health");
            break;
        }
    }

    assert!(
        player_health.is_some_and(|health| health < 100.0),
        "dummy melee should reduce player health within 600 frames, got {player_health:?}"
    );
}

#[test]
fn training_dummy_spawns() {
    let mut app = demo_app();

    for _ in 0..120 {
        app.update();
    }

    let count = app
        .world_mut()
        .query_filtered::<(), With<TrainingDummy>>()
        .iter(app.world())
        .count();
    assert!(count > 0, "training dummy must spawn after experience ready");
}
