# aa_scene — Subsystem Specification

> **Normative** | Priority P0

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-SCENE-001 | `PrefabAsset` MUST load from RON per `schemas/prefab.schema.json` |
| REQ-SCENE-002 | `spawn_prefab()` MUST create entity hierarchy with `Children`/`Parent` |
| REQ-SCENE-003 | `Possesses`/`PossessedBy` relationships MUST use Bevy 0.19 relationship API |
| REQ-SCENE-004 | `SceneAsset` MUST support inline entities and prefab references |
| REQ-SCENE-005 | Despawn MUST recursively despawn children unless `PersistOnParentDespawn` |
| REQ-SCENE-006 | Spawn MUST assign stable `SceneEntityId` for editor round-trip |
| REQ-SCENE-007 | Hot reload of scene RON in dev MUST not leak entities (full replace or diff) |
| REQ-SCENE-008 | Scene patch files MUST validate against `schemas/scene_patch.schema.json` |
| REQ-SCENE-009 | Patch application MUST address stable `SceneEntityId`, not transient ECS `Entity` IDs |
| REQ-SCENE-010 | Patch dry-run MUST compute affected files/entities without mutating source files |
| REQ-SCENE-011 | Patch application MUST produce an undo token sufficient to restore the previous scene asset state |
| REQ-SCENE-012 | Patch validation MUST reject absolute paths, parent-directory paths, and writes outside the project allowlist |

## API Contract

```rust
#[derive(Asset, TypePath)]
pub struct PrefabAsset { pub schema_version: u32, pub id: String, pub children: Vec<PrefabEntity> }

#[derive(Asset, TypePath)]
pub struct SceneAsset { pub schema_version: u32, pub id: String, pub entities: Vec<SceneEntity> }

pub fn spawn_prefab(commands: &mut Commands, prefab: &PrefabAsset, transform: Transform) -> Entity;
pub fn load_scene(commands: &mut Commands, scene: &SceneAsset) -> Vec<Entity>;

pub struct ScenePatch {
    pub schema_version: u32,
    pub id: String,
    pub target_path: String,
    pub ops: Vec<ScenePatchOp>,
}

pub enum ScenePatchOp {
    SpawnEntity { entity_id: String },
    DespawnEntity { entity_id: String, recursive: bool },
    InstantiatePrefab { entity_id: String, prefab: String },
    SetTransform { entity_id: String },
    AddComponent { entity_id: String, component: String },
    ReplaceComponent { entity_id: String, component: String },
    RemoveComponent { entity_id: String, component: String },
    SetDataLayer { entity_id: String, layer: String, enabled: bool },
}

pub struct ScenePatchReport {
    pub affected_files: Vec<String>,
    pub affected_entities: Vec<String>,
    pub undo_token: String,
}

#[derive(Component)]
#[relationship(relationship_target = PossessedBy)]
pub struct Possesses(pub Entity);

#[derive(Component)]
#[relationship(relationship_target = Possesses)]
pub struct PossessedBy(pub Entity);
```

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-SCENE-01 | Spawn prefab | hierarchy depth ≥ 1 | integration |
| T-SCENE-02 | Possession pair | bidirectional query works | unit |
| T-SCENE-03 | Scene round-trip | save/load equal entity count | integration |
| T-SCENE-04 | Patch schema | malformed patch rejected | unit |
| T-SCENE-05 | Patch dry-run | affected files/entities reported, no write | integration |
| T-SCENE-06 | Patch undo | restored scene equals original | integration |

## Acceptance: P0 when T-SCENE-01–06 green.
