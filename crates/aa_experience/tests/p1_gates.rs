//! P1 gate integration tests — experience boot (Gate P1-06).

use aa_ability::{AaAbilityPlugin, GameplayAbilityAsset};
use aa_core::{init_project, AaCorePlugin};
use aa_experience::{
    AaExperiencePlugin, ExperienceAction, ExperienceDefinitionAsset, ExperienceReady,
};
use bevy::prelude::*;

const DEMO_PROJECT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/demo_game");

fn demo_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, init_project(DEMO_PROJECT)));
    app.add_plugins(AaCorePlugin::default());
    app.add_plugins(AaAbilityPlugin);
    app.add_plugins(AaExperiencePlugin {
        default_experience: "experiences/demo".into(),
    });
    app
}

/// Gate P1-06 / T-EXP-01: default experience loads and `ExperienceReady` fires promptly.
#[test]
fn experience_load() {
    let mut app = demo_app();
    let max_frames = 300; // 5 s @ 60 Hz
    let mut ready_event = None;

    for frame in 0..max_frames {
        app.update();
        if let Some(event) = app
            .world_mut()
            .resource_mut::<Messages<ExperienceReady>>()
            .drain()
            .next()
        {
            ready_event = Some((frame, event));
            break;
        }
    }

    let (ready_frame, event) = ready_event.expect("ExperienceReady must fire after asset load");
    assert!(
        ready_frame < max_frames,
        "ExperienceReady should arrive within 5 s (frame {ready_frame})"
    );

    let granted_paths: Vec<String> = {
        let experiences = app.world().resource::<Assets<ExperienceDefinitionAsset>>();
        let experience = experiences
            .get(&event.handle)
            .expect("experience asset must be resident after ready");

        assert_eq!(experience.id, "experiences/demo");
        assert_eq!(experience.default_pawn, "prefabs/player");

        experience
            .actions
            .iter()
            .filter_map(|action| match action {
                ExperienceAction::GrantAbilitySet { abilities } => Some(abilities.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    };

    assert!(
        !granted_paths.is_empty(),
        "demo experience must declare GrantAbilitySet actions"
    );
    assert!(
        granted_paths.contains(&"abilities/fireball".to_string()),
        "combat demo must grant fireball"
    );

    let handles: Vec<(String, Handle<GameplayAbilityAsset>)> = {
        let asset_server = app.world().resource::<AssetServer>();
        granted_paths
            .iter()
            .map(|path| {
                (
                    path.clone(),
                    asset_server.load::<GameplayAbilityAsset>(format!("{path}.ron")),
                )
            })
            .collect()
    };

    for _ in 0..120 {
        app.update();
        let abilities = app.world().resource::<Assets<GameplayAbilityAsset>>();
        if handles
            .iter()
            .all(|(_, handle)| abilities.get(handle).is_some())
        {
            break;
        }
    }

    let abilities = app.world().resource::<Assets<GameplayAbilityAsset>>();
    for (path, handle) in handles {
        let asset = abilities
            .get(&handle)
            .unwrap_or_else(|| panic!("ability grant `{path}` must resolve after experience load"));
        assert_eq!(asset.id, path);
    }
}
