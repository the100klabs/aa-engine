//! P3-07 — gameplay effect RON hot reload completes within 500ms.

use std::path::Path;
use std::time::{Duration, Instant};

use aa_ability::{AaAbilityPlugin, GameplayEffectAsset};
use aa_core::init_project;
use bevy::prelude::*;

const EFFECT_INITIAL: &str = r#"GameplayEffect(
    schema_version: 1,
    id: "effects/test_hot_reload",
    duration: Instant,
    modifiers: [
        (attribute: "Health", op: Add, magnitude: -10.0),
    ],
    granted_tags: [],
    application_tags_required: [],
    application_tags_blocked: [],
    cues_on_apply: [],
)"#;

const EFFECT_RELOADED: &str = r#"GameplayEffect(
    schema_version: 1,
    id: "effects/test_hot_reload",
    duration: Instant,
    modifiers: [
        (attribute: "Health", op: Add, magnitude: -99.0),
    ],
    granted_tags: [],
    application_tags_required: [],
    application_tags_blocked: [],
    cues_on_apply: [],
)"#;

fn write_effect(project_root: &Path, contents: &str) {
    let effects_dir = project_root.join("assets/effects");
    std::fs::create_dir_all(&effects_dir).expect("create effects dir");
    std::fs::write(effects_dir.join("test_hot_reload.ron"), contents).expect("write effect ron");
}

fn wait_loaded(
    app: &mut App,
    handle: &Handle<GameplayEffectAsset>,
    timeout: Duration,
) -> GameplayEffectAsset {
    let started = Instant::now();
    loop {
        app.update();
        if let Some(asset) = app
            .world()
            .resource::<Assets<GameplayEffectAsset>>()
            .get(handle)
        {
            return asset.clone();
        }
        assert!(
            started.elapsed() < timeout,
            "timed out loading gameplay effect asset"
        );
    }
}

fn wait_magnitude(
    app: &mut App,
    handle: &Handle<GameplayEffectAsset>,
    expected: f32,
    timeout: Duration,
) {
    let started = Instant::now();
    loop {
        app.update();
        if let Some(asset) = app
            .world()
            .resource::<Assets<GameplayEffectAsset>>()
            .get(handle)
            && let Some(modifier) = asset.modifiers.first()
            && (modifier.magnitude - expected).abs() < f32::EPSILON
        {
            return;
        }
        assert!(
            started.elapsed() < timeout,
            "timed out waiting for reloaded magnitude {expected}"
        );
    }
}

#[test]
fn gameplay_effect_ron_hot_reload_under_500ms() {
    let temp = tempfile::tempdir().expect("temp project dir");
    write_effect(temp.path(), EFFECT_INITIAL);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, init_project(temp.path()), AaAbilityPlugin));

    let handle = {
        let server = app.world().resource::<AssetServer>();
        server.load("effects/test_hot_reload.ron")
    };

    let initial = wait_loaded(&mut app, &handle, Duration::from_secs(5));
    assert_eq!(initial.modifiers[0].magnitude, -10.0);

    std::fs::write(
        temp.path().join("assets/effects/test_hot_reload.ron"),
        EFFECT_RELOADED,
    )
    .expect("rewrite effect ron");

    let reload_started = Instant::now();
    app.world()
        .resource::<AssetServer>()
        .reload("effects/test_hot_reload.ron");
    wait_magnitude(&mut app, &handle, -99.0, Duration::from_secs(5));

    let elapsed = reload_started.elapsed();
    assert!(
        elapsed <= Duration::from_millis(500),
        "effect hot reload took {elapsed:?}, expected <= 500ms"
    );
}
