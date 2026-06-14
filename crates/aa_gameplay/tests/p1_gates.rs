//! P1 gate integration tests — pawn init FSM (Gate P1-07).

use aa_ability::{AaAbilityPlugin, AbilityRegistry, AttributeSet};
use aa_assets::AaAssetsPlugin;
use aa_core::{init_project, AaCorePlugin};
use aa_experience::AaExperiencePlugin;
use aa_gameplay::{AaGameplayPlugin, Pawn, PlayerState};
use aa_input::{ActiveInputContexts, InputMappingContextAsset, AaInputPlugin};
use aa_scene::PendingInit;
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

fn pending_init_flags(world: &mut World) -> (bool, bool) {
    let mut player_pending = false;
    let mut pawn_pending = false;

    for entity in world.query_filtered::<Entity, With<PlayerState>>().iter(world) {
        if world.get::<PendingInit>(entity).is_some() {
            player_pending = true;
        }
    }
    for entity in world.query_filtered::<Entity, With<Pawn>>().iter(world) {
        if world.get::<PendingInit>(entity).is_some() {
            pawn_pending = true;
        }
    }

    (player_pending, pawn_pending)
}

/// Gate P1-07 / T-GAME-05: init chain applies attribute set + input context, clears `PendingInit`.
#[test]
fn pawn_init_chain() {
    let mut app = demo_app();
    let max_frames = 300;
    let mut init_started_frame: Option<u32> = None;
    let mut init_done_frame: Option<u32> = None;

    for frame in 0..max_frames {
        app.update();

        let world = app.world_mut();
        let (player_pending, pawn_pending) = pending_init_flags(world);
        let has_player = !world
            .query_filtered::<(), With<PlayerState>>()
            .iter(world)
            .collect::<Vec<_>>()
            .is_empty();

        if init_started_frame.is_none() && (player_pending || pawn_pending) {
            init_started_frame = Some(frame);
        }

        if init_started_frame.is_some()
            && has_player
            && !player_pending
            && !pawn_pending
            && init_done_frame.is_none()
        {
            init_done_frame = Some(frame);
            break;
        }
    }

    let start = init_started_frame.expect("player/pawn init must enter PendingInit state");
    let done = init_done_frame.expect("init chain must finish within test budget");
    let init_frames = done - start;
    assert!(
        init_frames < 10,
        "PendingInit must clear within 10 frames (took {init_frames})"
    );

    let (health, has_stamina, grants_fireball) = {
        let world = app.world_mut();
        let mut player_attrs: Option<&AttributeSet> = None;
        let mut player_registry: Option<&AbilityRegistry> = None;
        for (entity, attrs, registry) in world
            .query::<(Entity, &AttributeSet, &AbilityRegistry)>()
            .iter(world)
        {
            if world.get::<PlayerState>(entity).is_some() {
                player_attrs = Some(attrs);
                player_registry = Some(registry);
                break;
            }
        }

        let attrs = player_attrs.expect("PlayerState must have AttributeSet after init");
        let registry = player_registry.expect("PlayerState must retain AbilityRegistry");
        (
            attrs.get("Health").unwrap_or(0.0),
            attrs.get("Stamina").is_some(),
            registry
                .granted
                .iter()
                .any(|g| g.ability_id == "abilities/fireball"),
        )
    };

    assert!(
        (health - 100.0).abs() <= 0.01,
        "hero_combat attribute set must apply Health=100"
    );
    assert!(has_stamina, "hero_combat attribute set must include Stamina");
    assert!(
        grants_fireball,
        "experience GrantAbilitySet must populate ability registry"
    );

    let shooter_loaded = {
        let world = app.world_mut();
        let contexts = world.resource::<ActiveInputContexts>();
        assert!(
            !contexts.contexts.is_empty(),
            "AddInputContext action must push shooter mapping context"
        );
        let mapping_assets = world.resource::<Assets<InputMappingContextAsset>>();
        contexts.contexts.iter().any(|handle| {
            mapping_assets
                .get(handle)
                .is_some_and(|ctx| ctx.id == "input/contexts/shooter")
        })
    };
    assert!(shooter_loaded, "shooter input context asset must resolve");

    let (player_pending, pawn_pending) = pending_init_flags(app.world_mut());
    assert!(!player_pending, "PlayerState must not retain PendingInit");
    assert!(!pawn_pending, "Pawn must not retain PendingInit");
}
