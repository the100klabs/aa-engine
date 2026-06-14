//! Gate P0-04 — AA schedule graph must have zero system-order ambiguities.

use aa_ability::AaAbilityPlugin;
use aa_assets::AaAssetsPlugin;
use aa_core::{init_project, AaCorePlugin};
use aa_experience::AaExperiencePlugin;
use aa_gameplay::AaGameplayPlugin;
use aa_input::AaInputPlugin;
use aa_scene::AaScenePlugin;
use aa_tags::AaTagsPlugin;
use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings, Schedules};
use bevy::input::InputPlugin;
use bevy::prelude::*;

const DEMO_PROJECT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/demo_game");

/// Headless AA plugin stack (demo_game minus rendering/window and game-specific systems).
fn representative_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, init_project(DEMO_PROJECT)));
    app.add_plugins(AaCorePlugin::default());
    app.add_plugins(AaAssetsPlugin);
    app.add_plugins(AaTagsPlugin);
    app.add_plugins(AaInputPlugin);
    app.add_plugins(AaAbilityPlugin);
    app.add_plugins(AaExperiencePlugin {
        default_experience: "experiences/demo".into(),
    });
    app.add_plugins(AaScenePlugin);
    app.add_plugins(AaGameplayPlugin);
    app
}

/// Builds every registered schedule with ambiguity detection enabled.
fn assert_zero_schedule_ambiguities(app: &mut App) {
    app.configure_schedules(ScheduleBuildSettings {
        ambiguity_detection: LogLevel::Error,
        hierarchy_detection: LogLevel::Ignore,
        ..default()
    });
    app.finish();
    app.cleanup();

    // One update runs startup + main schedules (including FixedUpdate via RunFixedMainLoop).
    // With `ambiguity_detection: Error`, any conflicting systems panic during initialize.
    let update_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        app.update();
    }));

    if let Err(payload) = update_result {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "schedule ambiguity panic (non-string payload)".into());

        let schedule_count = app.world().resource::<Schedules>().iter().count();
        panic!(
            "expected 0 schedule ambiguities across {schedule_count} registered schedules; \
             app.update() failed while building schedules:\n{message}"
        );
    }
}

/// Gate P0-04: representative AA plugin stack must produce zero schedule ambiguities.
#[test]
fn schedule_ambiguity() {
    let mut app = representative_app();
    assert_zero_schedule_ambiguities(&mut app);
}
