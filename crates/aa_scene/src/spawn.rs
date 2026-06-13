use bevy::prelude::*;

use crate::components::PendingInit;
use crate::prefab::{PrefabAsset, PrefabEntity};
use crate::ron_components::{apply_components, apply_entity_name};
use crate::scene::{SceneAsset, SceneEntity};

pub fn spawn_prefab(commands: &mut Commands, prefab: &PrefabAsset, transform: Transform) -> Entity {
    let root = commands
        .spawn((transform, PendingInit, Name::new(prefab.id.clone())))
        .id();

    for child in &prefab.children {
        spawn_prefab_entity(commands, child, root);
    }

    root
}

fn spawn_prefab_entity(commands: &mut Commands, entity: &PrefabEntity, parent: Entity) -> Entity {
    let mut entity_commands = commands.spawn((ChildOf(parent), PendingInit));
    apply_entity_name(&mut entity_commands, entity.name.as_deref());
    apply_components(&mut entity_commands, &entity.components);
    let id = entity_commands.id();

    for child in &entity.children {
        spawn_prefab_entity(commands, child, id);
    }

    id
}

pub fn load_scene(commands: &mut Commands, scene: &SceneAsset) -> Vec<Entity> {
    scene
        .entities
        .iter()
        .map(|entity| spawn_scene_entity(commands, entity, None))
        .collect()
}

fn spawn_scene_entity(
    commands: &mut Commands,
    entity: &SceneEntity,
    parent: Option<Entity>,
) -> Entity {
    let mut entity_commands = commands.spawn(PendingInit);

    if let Some(parent) = parent {
        entity_commands.insert(ChildOf(parent));
    }

    apply_entity_name(&mut entity_commands, entity.name.as_deref());
    apply_components(&mut entity_commands, &entity.components);

    if let Some(prefab_path) = &entity.prefab {
        warn!(
            "scene entity references prefab `{prefab_path}` — resolve via AssetServer in Phase 1"
        );
    }

    let id = entity_commands.id();

    for child in &entity.children {
        spawn_scene_entity(commands, child, Some(id));
    }

    id
}
