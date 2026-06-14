//! P0 gate integration tests — prefab spawn (Gate P0-03).

use aa_core::init_project;
use aa_scene::{spawn_prefab, AaScenePlugin, PendingInit, PrefabAsset};
use bevy::prelude::*;

const DEMO_PROJECT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/demo_game");
const PLAYER_PREFAB: &str = "prefabs/player";

fn demo_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, init_project(DEMO_PROJECT)));
    app.add_plugins(AaScenePlugin);
    app
}

fn entity_component_count(world: &World, entity: Entity) -> usize {
    world
        .get_entity(entity)
        .expect("spawned entity must exist")
        .archetype()
        .components()
        .len()
}

/// Gate P0-03 / T-SCENE-01: demo player prefab loads and spawns with ≥ 3 components.
#[test]
fn spawn_player_prefab() {
    let mut app = demo_app();
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<PrefabAsset>(format!("{PLAYER_PREFAB}.ron"));

    for _ in 0..120 {
        app.update();
        let prefabs = app.world().resource::<Assets<PrefabAsset>>();
        if prefabs.get(&handle).is_some() {
            break;
        }
    }

    let prefab = app
        .world()
        .resource::<Assets<PrefabAsset>>()
        .get(&handle)
        .unwrap_or_else(|| panic!("`{PLAYER_PREFAB}` must resolve from demo_game assets"))
        .clone();

    assert_eq!(prefab.id, "prefabs/player");
    assert!(
        !prefab.children.is_empty(),
        "player prefab must declare at least one child entity"
    );

    let spawned = {
        let mut commands = app.world_mut().commands();
        spawn_prefab(&mut commands, &prefab, Transform::default())
    };
    app.update();

    let world = app.world_mut();
    let root_components = entity_component_count(world, spawned);
    assert!(
        root_components >= 3,
        "prefab root must have ≥ 3 components (got {root_components})"
    );

    assert!(
        world.get::<PendingInit>(spawned).is_some(),
        "spawn_prefab must tag root with PendingInit"
    );
    assert!(
        world.get::<Transform>(spawned).is_some(),
        "spawn_prefab must attach root Transform"
    );
    assert!(
        world.get::<Name>(spawned).is_some(),
        "spawn_prefab must attach root Name from prefab id"
    );

    let mut child_found = false;
    for child in world
        .query_filtered::<Entity, With<ChildOf>>()
        .iter(world)
    {
        child_found = true;
        let child_components = entity_component_count(world, child);
        assert!(
            child_components >= 3,
            "prefab child must have ≥ 3 components (got {child_components})"
        );
    }
    assert!(child_found, "player prefab must spawn at least one child entity");
}
