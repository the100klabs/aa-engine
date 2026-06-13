# 13 — Data Schemas

> **Text-first, agent-friendly asset formats.** All schemas are original — not UE `.uasset` translations. Use RON for Rust-native assets, TOML for config.

## Design Rules

1. **Every gameplay asset is text** (RON/TOML) unless binary is required (mesh, texture, audio)
2. **Stable IDs** — string paths as primary keys (`"abilities/fireball"`)
3. **Schema version field** on every asset: `schema_version: 1`
4. **Soft references** — string paths, resolved at load via `AssetRegistry`
5. **Validation** — `aa_cli validate` checks all refs before play

---

## Project Manifest

**File:** `aa.project.toml`  
**UE analog:** `.uproject` + `DefaultGame.ini` fragments
**Normative schema:** `docs/specs/schemas/project.schema.json`

```toml
schema_version = 1
name = "my_aa_game"
version = "0.1.0"

[engine]
config_root = "config"
assets_root = "assets"
default_experience = "experiences/shooter_dm"
startup_scene = "scenes/main_menu"

[plugins]
runtime = [
    "aa_core",
    "aa_gameplay",
    "aa_ability",
]

[features]
# Compile-time game feature crates (Lyra GameFeatures equivalent)
enabled = ["feature_shooter_core"]

[build]
default_binary = "aa_game"
```

`default_experience` and `startup_scene` are optional during early project scaffolding. When present, validation must treat them as soft references and reject missing targets before play.

---

## Config Layers

### `config/engine.toml` (→ `DefaultEngine.ini`)

```toml
[app]
window_title = "AA Game"
target_fps = 60

[render]
resolution_scale = 1.0
msaa = 4
shadow_quality = "high"  # low | medium | high | epic

[audio]
master_volume = 1.0

[net]
default_port = 7777
max_players = 16
```

### `config/game.toml` (→ `DefaultGame.ini`)

```toml
[gameplay]
default_damage_scale = 1.0
respawn_delay_secs = 3.0

[teams]
max_teams = 4
friendly_fire = false
```

### `config/input.toml` (→ `DefaultInput.ini` + Enhanced Input)

```toml
[defaults]
mouse_sensitivity = 1.0
gamepad_deadzone = 0.15

# Mapping contexts loaded at runtime; see input contexts below
```

### `config/scalability.toml` (→ `BaseScalability.ini`)

```toml
[presets.low]
render.resolution_scale = 0.5
render.shadow_quality = "low"
render.gi = false

[presets.medium]
render.resolution_scale = 0.75
render.shadow_quality = "medium"

[presets.high]
render.resolution_scale = 1.0
render.shadow_quality = "high"

[presets.epic]
render.resolution_scale = 1.0
render.shadow_quality = "epic"
render.gi = true
```

### Config merge order

```
config/engine_base.toml      # shipped with aa_engine
  ← config/engine.toml
    ← config/platforms/{os}.toml
      ← config/game.toml
        ← config/user.toml   # gitignored
          ← CLI --set net.port=8888
```

---

## Tag Dictionary

**File:** `assets/data/tags.ron`  
**UE analog:** Gameplay Tag Dictionary

```ron
TagDictionary(
    schema_version: 1,
    tags: [
        // States
        "State.Stunned",
        "State.Dead",
        "State.Blocking",
        // Abilities
        "Ability.Cooldown.Fireball",
        "Ability.Cooldown.Dash",
        // Damage types
        "Damage.Fire",
        "Damage.Physical",
        // Cues
        "GameplayCue.Fire.Impact",
        "GameplayCue.Hit.Physical",
    ],
)
```

---

## Experience Definition

**File:** `assets/experiences/shooter_dm.ron`  
**UE analog:** `ULyraExperienceDefinition`  
**Local ref:** `Samples/Games/Lyra/.../LyraExperienceDefinition.h`

```ron
ExperienceDefinition(
    schema_version: 1,
    id: "experiences/shooter_dm",
    display_name: "Shooter Deathmatch",
    game_features: ["feature_shooter_core"],
    default_pawn: "pawns/hero_shooter",
    action_sets: ["action_sets/shooter_base"],
    actions: [
        GrantAbilitySet(abilities: ["abilities/rifle_primary", "abilities/dash"]),
        AddInputContext(context: "input/contexts/shooter"),
        RegisterUiLayout(layout: "ui/hud_shooter"),
    ],
)
```

---

## Game Feature Action Types

**File:** `assets/action_sets/shooter_base.ron`

```ron
ActionSet(
    schema_version: 1,
    id: "action_sets/shooter_base",
    actions: [
        GrantAbilitySet(abilities: ["abilities/melee"]),
        AddTags(tags: ["Game.Mode.Shooter"]),
    ],
)
```

### Action enum (schema)

| Variant | Fields | UE analog |
|---------|--------|-----------|
| `GrantAbilitySet` | `abilities: [AbilityId]` | GameFeatureAction_AddAbilities |
| `AddInputContext` | `context: InputContextId` | input mapping setup |
| `RegisterUiLayout` | `layout: UiLayoutId` | widget layer setup |
| `AddTags` | `tags: [Tag]` | grant loose tags |
| `LoadScene` | `scene: SceneId` | map load |

---

## Pawn Data

**File:** `assets/pawns/hero_shooter.ron`  
**UE analog:** `ULyraPawnData`

```ron
PawnData(
    schema_version: 1,
    id: "pawns/hero_shooter",
    display_name: "Hero Shooter",
    mesh: "meshes/characters/hero.glb",
    physics: PhysicsProfile(
        capsule_height: 1.8,
        capsule_radius: 0.4,
        max_walk_speed: 6.0,
        jump_velocity: 8.0,
    ),
    animation: AnimationProfile(
        graph: "animation/graphs/hero_locomotion",
        skeleton: "animation/skeletons/hero",
    ),
    attribute_sets: ["attributes/hero_combat"],
    default_tags: ["Faction.Player"],
)
```

---

## Attribute Set

**File:** `assets/attributes/hero_combat.ron`
**Normative schema:** `docs/specs/schemas/attribute_set.schema.json`

```ron
AttributeSet(
    schema_version: 1,
    id: "attributes/hero_combat",
    attributes: [
        (name: "Health", default: 100.0, min: 0.0, max: 100.0, replicated: true),
        (name: "Stamina", default: 100.0, min: 0.0, max: 100.0, replicated: true),
        (name: "DamageMultiplier", default: 1.0, replicated: false),
    ],
)
```

---

## Gameplay Effect

**File:** `assets/effects/burning.ron`  
**UE analog:** `UGameplayEffect`

```ron
GameplayEffect(
    schema_version: 1,
    id: "effects/burning",
    duration: Periodic(seconds: 1.0, count: 5),
    modifiers: [
        (attribute: "Health", op: Add, magnitude: -5.0),
    ],
    granted_tags: ["State.Debuff.Burning"],
    application_tags_required: [],  // target must have these
    application_tags_blocked: ["State.Dead"],
    cues_on_apply: ["GameplayCue.Fire.Impact"],
)
```

### Modifier ops

| Op | UE analog | Aggregation |
|----|-----------|-------------|
| `Add` | Additive | Sum |
| `Multiply` | Multiply | Sum of % then apply |
| `Override` | Override | Last wins |

---

## Gameplay Ability

**File:** `assets/abilities/fireball.ron`

```ron
GameplayAbility(
    schema_version: 1,
    id: "abilities/fireball",
    display_name: "Fireball",
    cooldown_tags: ["Ability.Cooldown.Fireball"],
    activation_tags_required: [],
    activation_tags_blocked: ["State.Stunned", "State.Dead"],
    cost_effect: None,  // or Some("effects/stamina_cost")
    montage: Some("animation/montages/cast_fire"),
    cue_on_activate: Some("GameplayCue.Fire.Cast"),
    impl: "fireball",  // maps to Rust ability registrar key
)
```

**MVP:** `impl` string keys into Rust `AbilityRegistry`.  
**AA:** optional WASM/script module path.

---

## Input Action + Mapping Context

**File:** `assets/input/actions.ron`

```ron
InputActions(
    schema_version: 1,
    actions: [
        (id: "Move", value_type: Axis2D),
        (id: "Look", value_type: Axis2D),
        (id: "Jump", value_type: Digital),
        (id: "Fire", value_type: Digital),
        (id: "Ability1", value_type: Digital),
    ],
)
```

**File:** `assets/input/contexts/shooter.ron`

```ron
InputMappingContext(
    schema_version: 1,
    id: "input/contexts/shooter",
    priority: 0,
    mappings: [
        (action: "Move", bindings: [GamepadLeftStick, WASD]),
        (action: "Look", bindings: [MouseDelta, GamepadRightStick]),
        (action: "Jump", bindings: [KeyboardSpace, GamepadSouth]),
        (action: "Fire", bindings: [MouseLeft, GamepadRightTrigger]),
        (action: "Ability1", bindings: [KeyboardQ, GamepadLeftShoulder]),
    ],
)
```

---

## Prefab / Scene

**File:** `assets/scenes/arena_01.ron`

```ron
Scene(
    schema_version: 1,
    id: "scenes/arena_01",
    entities: [
        (
            name: "SpawnPoint_01",
            components: {
                "Transform": (translation: (0.0, 0.0, 0.0)),
                "SpawnPoint": (team: 0, tag: "Spawn.Player"),
            },
        ),
        (
            name: "Floor",
            prefab: "prefabs/environment/floor_tile",
            components: {
                "Transform": (translation: (0.0, 0.0, 0.0), scale: (10.0, 1.0, 10.0)),
            },
        ),
    ],
)
```

**File:** `assets/prefabs/environment/floor_tile.ron`

```ron
Prefab(
    schema_version: 1,
    id: "prefabs/environment/floor_tile",
    children: [
        (
            components: {
                "Mesh": (path: "meshes/floor.glb"),
                "Collider": (shape: Box, size: (1.0, 0.1, 1.0)),
            },
        ),
    ],
)
```

---

## World Sector (streaming)

**File:** `assets/sectors/cell_3_5.ron`  
**UE analog:** World Partition cell descriptor

```ron
SectorDescriptor(
    schema_version: 1,
    id: "sectors/cell_3_5",
    coord: (3, 5),
    bounds: Aabb(
        min: (768.0, 0.0, 1280.0),
        max: (1024.0, 256.0, 1536.0),
    ),
    data_layers: ["Base"],
    entities: [
        (prefab: "prefabs/buildings/shop_a", transform: (...)),
        (prefab: "prefabs/props/barrel", transform: (...)),
    ],
    navmesh: Some("nav/cell_3_5.bin"),
    hlod: None,  // AA: Some("hlod/cell_3_5.glb")
)
```

---

## Data Layer Definition

**File:** `assets/data_layers.ron`

```ron
DataLayerRegistry(
    schema_version: 1,
    layers: [
        (id: "Base", default_state: Active),
        (id: "NightEmissives", default_state: Loaded),
        (id: "DestroyedBridge", default_state: Unloaded),
    ],
)
```

---

## AI Profile

**File:** `assets/ai/bandit_guard.ron`
**Normative schema:** `docs/specs/schemas/ai_profile.schema.json`

```ron
AiProfile(
    schema_version: 1,
    id: "ai/bandit_guard",
    display_name: "Bandit Guard",
    behavior: (kind: "guard", home_radius_m: 20.0, engage_radius_m: 35.0),
    perception: (sight_radius_m: 45.0, hearing_radius_m: 20.0, field_of_view_degrees: 120.0),
    combat: (preferred_range_m: 12.0, abilities: ["abilities/rifle_primary"]),
    lod: (full_radius_m: 80.0, simplified_radius_m: 200.0, dormant_radius_m: 500.0),
    tags: ["Faction.Bandit"],
)
```

---

## Spawn Table

**File:** `assets/spawn_tables/bandit_camp.ron`
**Normative schema:** `docs/specs/schemas/spawn_table.schema.json`

```ron
SpawnTable(
    schema_version: 1,
    id: "spawn_tables/bandit_camp",
    display_name: "Bandit Camp",
    data_layers: ["gameplay"],
    entries: [
        (
            id: "bandit_guard",
            pawn: "pawns/bandit_guard",
            ai_profile: "ai/bandit_guard",
            weight: 1.0,
            count_min: 2,
            count_max: 5,
            spawn_radius_m: 18.0,
        ),
    ],
    budgets: (max_alive: 12, max_spawn_per_activation: 6, memory_budget_mb: 32.0),
)
```

---

## Asset Manifest

**File:** `assets/asset_manifest.json` (generated by `aa_cli import`)
**Normative schema:** `docs/specs/schemas/asset_manifest.schema.json`

```json
{
  "schema_version": 1,
  "generated_at": "2026-06-13T12:00:00Z",
  "assets": [
    {
      "id": "meshes/hero.glb",
      "kind": "mesh",
      "path": "meshes/hero.glb",
      "hash": "sha256:abc123",
      "deps": []
    },
    {
      "id": "abilities/fireball.ron",
      "kind": "ability",
      "path": "abilities/fireball.ron",
      "hash": "sha256:def456",
      "deps": ["effects/burning.ron", "animation/montages/cast_fire"]
    }
  ]
}
```

---

## Network Replication Manifest

**File:** `config/replication.toml`  
**UE analog:** Iris filter config in Lyra `DefaultEngine.ini`

```toml
[replication]
tick_rate = 60
max_bandwidth_kbps = 128

[components]
# All players receive
always_relevant = ["PlayerState", "GameState"]

[components.Pawn]
filter = "spatial"
radius_meters = 80.0

[components.Projectile]
filter = "spatial"
radius_meters = 100.0

[components.Pickup]
filter = "spatial"
radius_meters = 50.0
```

---

## Session / Editor State

**File:** `.aa/session.toml` (gitignored, editor only)

```toml
last_scene = "scenes/arena_01"
camera_position = [12.0, 5.0, 8.0]
camera_target = [0.0, 0.0, 0.0]
selected_entity = "entity_42"
```

---

## Schema Versioning Policy

| Change type | Action |
|-------------|--------|
| Add optional field | Bump minor; old assets still load |
| Rename field | Migration in `aa_assets::migrate` |
| Breaking layout | Bump `schema_version`; write migrator |

```rust
// Conceptual migrator
fn migrate_experience(mut doc: RonValue) -> RonValue {
    match doc.get("schema_version") {
        1 => doc,
        _ => panic!("unsupported"),
    }
}
```

---

## Validation Rules (`aa_cli validate`)

| Rule | Error code |
|------|------------|
| Unknown tag in effect | `TAG_UNKNOWN` |
| Ability refs missing effect | `REF_MISSING` |
| Sector bounds overlap inconsistent | `SECTOR_BOUNDS` |
| Cyclic prefab reference | `CYCLE_PREFAB` |
| Mesh path not in manifest | `ASSET_MISSING` |
| Replication component unregistered | `NET_UNREGISTERED` |

Output: SARIF 2.1 for agent consumption.

---

## Folder Layout (canonical project)

```
my_game/
├── aa.project.toml
├── Cargo.toml
├── config/
│   ├── engine.toml
│   ├── game.toml
│   ├── input.toml
│   ├── scalability.toml
│   └── replication.toml
├── assets/
│   ├── data/
│   │   └── tags.ron
│   ├── experiences/
│   ├── action_sets/
│   ├── pawns/
│   ├── attributes/
│   ├── abilities/
│   ├── effects/
│   ├── input/
│   ├── prefabs/
│   ├── scenes/
│   ├── sectors/
│   ├── meshes/          # binary
│   ├── textures/        # binary
│   └── asset_manifest.json
├── src/
│   └── main.rs
└── .aa/
    └── session.toml
```

---

*Schemas are architectural contracts. Implement loaders in `aa_assets` crate per type.*
