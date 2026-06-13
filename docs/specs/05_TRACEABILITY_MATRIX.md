# 05 — Traceability Matrix

> **Normative.** Maps UE5 responsibilities → AA requirements → crate → tests.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Mapped + test required |
| 🔄 | Partial / MVP subset |
| — | Explicit non-goal |

## Foundation

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `Core/Public/Misc/ConfigHierarchy.h` | Layered config | `aa_core` | REQ-CORE-010–015 | `config_merge_order` |
| `UnrealBuildTool` | Module graph | workspace | REQ-GLOBAL-090 | `workspace_build` |
| `CoreUObject/Object.h` | Object identity | — | REQ-GLOBAL-001 | ECS lint |
| `AssetRegistry` | Asset discovery | `aa_assets` | REQ-ASSET-001–020 | `validate_refs` |

## Object / Gameplay Model

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `Actor.h` | World presence | `aa_scene` | REQ-SCENE-001–030 | `spawn_prefab` |
| `Pawn.h` / `Controller.h` | Possession | `aa_gameplay` | REQ-GAME-010–025 | `possession_chain` |
| `GameModeBase.h` | Server rules | `aa_gameplay` | REQ-GAME-030–040 | `game_mode_spawn` |
| `PlayerState.h` | Persistent player | `aa_gameplay` | REQ-GAME-050–060 | `asc_on_player_state` |
| Bevy Relationships | Possession links | `aa_scene` | REQ-GLOBAL-012 | `relationship_pair` |

## Gameplay Framework

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `GameplayAbilities/README.md` | ASC gateway | `aa_ability` | REQ-ABILITY-001–080 | `effect_application` |
| `GameplayAbilities/AttributeSet.h` | Attributes | `aa_ability` | REQ-ABILITY-020–035 | `attribute_aggregate` |
| `GameplayTags` | Tag queries | `aa_tags` | REQ-TAG-001–020 | `tag_query_has_all` |
| `EnhancedInput` | Mapping contexts | `aa_input` | REQ-INPUT-001–025 | `context_stack_priority` |
| `LyraExperienceDefinition.h` | Experience | `aa_experience` | REQ-EXP-001–020 | `experience_load` |
| `LyraExperienceManagerComponent.h` | Load FSM | `aa_experience` | REQ-EXP-010–015 | `experience_state_machine` |
| `LyraPawnExtensionComponent.h` | Init states | `aa_gameplay` | REQ-GAME-070–080 | `pawn_init_chain` |

## World

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `WorldPartition/WorldPartition.h` | Sector streaming | `aa_world_stream` | REQ-STREAM-001–040 | `sector_cross` |
| `DataLayer/DataLayerManager.h` | Data layers | `aa_world_stream` | REQ-STREAM-030–035 | `layer_toggle` |
| `MassEntity/MassEntityManager.h` | Crowds | `aa_crowd` | REQ-STREAM-050–060 | `crowd_1000_agents` |
| `NavigationSystem` | Navmesh | `aa_nav` | REQ-STREAM-040 | `nav_path_find` |

## Rendering

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `Renderer/Private/Nanite/` | Virtual geometry | `aa_render` | REQ-RENDER-040 | `vg_render` 🔄 |
| `Renderer/Private/Lumen/` | Dynamic GI | `aa_render` | REQ-RENDER-030 | — (probes AA) |
| `Scalability.h` | Quality tiers | `aa_render` | REQ-RENDER-010–015 | `scalability_preset` |
| Bevy `bevy_pbr` | PBR baseline | Bevy | — | visual sign-off |

## Animation

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `AnimInstance` | Skeletal eval | `aa_animation` | REQ-ANIM-001–030 | `locomotion_fsm` |
| `PoseSearch` | Motion matching | `aa_animation` | REQ-ANIM-040–050 | `motion_match_search` 🔄 |
| `ControlRig` | Procedural rig | `aa_animation` | — | post-AA |

## Physics

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `CharacterMovementComponent.h` | Character move | `aa_physics` | REQ-PHY-001–040 | `character_walk_jump` |
| `PhysScene_Chaos.h` | Physics scene | `aa_physics` | REQ-PHY-010–015 | `rapier_step` |
| `NetworkPhysicsComponent.h` | Net physics | `aa_physics` + `aa_net` | REQ-PHY-050 | `physics_snapshot` 🔄 |

## Networking

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `NetDriver.h` | Transport | `aa_net` | REQ-NET-001–010 | `connect_8_clients` |
| `ReplicationGraph.h` | Relevancy | `aa_net` | REQ-NET-020–035 | `spatial_relevancy` |
| `Iris/ReplicationSystem.h` | Modern replication | `aa_net` | REQ-NET-040–055 | `component_replicate` |
| GAS prediction | Ability predict | `aa_ability` + `aa_net` | REQ-ABILITY-060–070 | `predict_fire_cancel` |

## Tooling / Studio

| UE5 source | Responsibility | Crate | REQ IDs | Test |
|------------|----------------|-------|---------|------|
| `UnrealEd` | Editor shell | `aa_editor` | REQ-EDIT-001–030 | `editor_save_scene` |
| `PythonScriptPlugin` | Automation | `aa_cli` + `aa_agent` | REQ-CLI-001–050 | `agent_add_ability` |
| `DataValidation` | Asset validation | `aa_cli` | REQ-CLI-020–030 | `validate_sarif` |
| Niagara editor | VFX graph | `aa_vfx` | — | post-AA |

## Coverage Targets

| Phase | REQ count target | Automated test target |
|-------|------------------|----------------------|
| P0 | 40 | 40 |
| P1 | 120 | 100 |
| P2 | 200 | 180 |
| P3 | 250 | 220 |
| AA | 280+ | 95%+ |

## Maintenance

When adding `REQ-{CRATE}-{NNN}` to any SPEC, add a row here in the same PR.
