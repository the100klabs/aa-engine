# 15 — Phase 0 Bootstrap Guide

> **Copy-paste starting point.** Follow this sequentially to create the `aa_engine` workspace. Estimated: 1–2 weeks solo.

## Prerequisites

| Tool | Version |
|------|---------|
| Rust | 1.85+ (edition 2024) |
| Bevy | 0.16+ (pin exact in workspace) |
| cargo-watch | optional, for hot reload |

---

## Step 1 — Create Workspace

```bash
mkdir aa_engine && cd aa_engine
```

### Root `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/aa_core",
    "crates/aa_reflect",
    "crates/aa_assets",
    "crates/aa_scene",
    "crates/aa_cli",
    "examples/demo_game",
]
default-members = ["examples/demo_game"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
bevy = { version = "0.16", default-features = true }
serde = { version = "1", features = ["derive"] }
ron = "0.8"
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "2"
clap = { version = "4", features = ["derive"] }
```

---

## Step 2 — `aa_core` Crate

```bash
cargo new --lib crates/aa_core
```

### `crates/aa_core/Cargo.toml`

```toml
[package]
name = "aa_core"
version.workspace = true
edition.workspace = true

[dependencies]
bevy = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }

[features]
default = []
editor = []
server = []
dev = ["editor"]
```

### `crates/aa_core/src/lib.rs` (skeleton)

```rust
pub mod config;
pub mod cvar;
pub mod plugin;
pub mod schedule;
pub mod role;

pub use plugin::AaCorePlugin;
pub use role::AppRole;
pub use schedule::AaSchedule;
```

### `crates/aa_core/src/role.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppRole {
    Client,
    DedicatedServer,
    ListenServer,
    Editor,
}
```

### `crates/aa_core/src/config.rs`

Implement layered TOML merge per `13_data_schemas.md`:

```rust
pub struct ConfigProvider {
    merged: toml::Table,
}

impl ConfigProvider {
    pub fn load(project_root: &std::path::Path) -> Result<Self, ConfigError> {
        // 1. engine_base.toml (include_bytes in crate)
        // 2. config/engine.toml
        // 3. config/game.toml
        // 4. config/user.toml if exists
        todo!("implement merge")
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        todo!("dot-path lookup")
    }
}
```

### `crates/aa_core/src/schedule.rs`

Register `AaSchedule` sets from `14_system_schedule_spec.md`.

### `crates/aa_core/src/plugin.rs`

```rust
use bevy::prelude::*;

pub struct AaCorePlugin;

impl Plugin for AaCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::role::AppRole>()
            .configure_sets(Update, (
                crate::schedule::AaSchedule::InitState,
                crate::schedule::AaSchedule::Animation,
            ).chain());
    }
}
```

**Exit test:** `cargo check -p aa_core` passes.

---

## Step 3 — `aa_reflect` Crate

```bash
cargo new --lib crates/aa_reflect
```

Minimum: `#[derive(Reflect)]` re-export + `PropertyInfo` registry stub.

```rust
pub use bevy::reflect::Reflect;

pub struct ReflectRegistry {
    // TypeId → serialized schema for editor/agent
}
```

---

## Step 4 — `aa_assets` Crate

```bash
cargo new --lib crates/aa_assets
```

Responsibilities:
- `AssetManifest` resource (load `asset_manifest.json`)
- RON loader for `TagDictionary`
- glTF via Bevy built-in

```rust
pub struct AaAssetsPlugin;

impl Plugin for AaAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetRegistry>()
            .add_systems(Startup, load_manifest);
    }
}
```

---

## Step 5 — `aa_scene` Crate

```bash
cargo new --lib crates/aa_scene
```

### Key types

```rust
#[derive(Asset, TypePath)]
pub struct PrefabAsset {
    pub children: Vec<PrefabEntity>,
}

#[derive(Component)]
pub struct PendingInit;

pub fn spawn_prefab(
    commands: &mut Commands,
    prefab: &PrefabAsset,
    transform: Transform,
) -> Entity {
    todo!("spawn hierarchy")
}
```

### Possession relationships (Bevy 0.16+)

```rust
#[derive(Component)]
#[relationship(relationship_target = PossessedBy)]
pub struct Possesses(pub Entity);

#[derive(Component)]
#[relationship(relationship_target = Possesses)]
pub struct PossessedBy(pub Entity);
```

---

## Step 6 — `aa_cli` Crate

```bash
cargo new --bin crates/aa_cli
```

### Commands (Phase 0)

```rust
// clap structure
enum Commands {
    New { name: String },
    Run { project: PathBuf },
    Check { project: PathBuf },
    Validate { project: PathBuf },
}
```

```bash
cargo run -p aa_cli -- new demo_game
cargo run -p aa_cli -- check .
cargo run -p aa_cli -- validate .
```

---

## Step 7 — Example Game

```bash
cargo new --bin examples/demo_game
```

### `examples/demo_game/Cargo.toml`

```toml
[package]
name = "demo_game"
version.workspace = true
edition.workspace = true

[dependencies]
bevy = { workspace = true }
aa_core = { path = "../../crates/aa_core" }
aa_assets = { path = "../../crates/aa_assets" }
aa_scene = { path = "../../crates/aa_scene" }
```

### `examples/demo_game/aa.project.toml`

Copy from `13_data_schemas.md`.

### `examples/demo_game/src/main.rs`

```rust
use aa_core::AaCorePlugin;
use aa_assets::AaAssetsPlugin;
use aa_scene::AaScenePlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AaCorePlugin)
        .add_plugins(AaAssetsPlugin)
        .add_plugins(AaScenePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // TODO: spawn prefab from RON
}
```

---

## Step 8 — Config Files

Create in `examples/demo_game/config/`:

- `engine.toml`
- `game.toml`
- `scalability.toml`

Copy contents from `13_data_schemas.md`.

---

## Step 9 — First Prefab

`examples/demo_game/assets/prefabs/player.ron`:

```ron
Prefab(
    schema_version: 1,
    id: "prefabs/player",
    children: [
        (
            components: {
                "Transform": (translation: (0.0, 1.0, 0.0)),
                "Name": "Player",
            },
        ),
    ],
)
```

Wire loader in `aa_scene` to spawn on `Startup`.

---

## Step 10 — AGENTS.md

Copy `docs/research/unreal_to_bevy/AGENTS.md` to project root (see that file).

---

## Step 11 — CI Stub

`.github/workflows/ci.yml`:

```yaml
name: ci
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check --workspace
      - run: cargo run -p aa_cli -- validate examples/demo_game
```

---

## Directory Target (end of Phase 0)

```
aa_engine/
├── Cargo.toml
├── AGENTS.md
├── crates/
│   ├── aa_core/
│   ├── aa_reflect/
│   ├── aa_assets/
│   ├── aa_scene/
│   └── aa_cli/
├── examples/
│   └── demo_game/
│       ├── aa.project.toml
│       ├── config/
│       ├── assets/
│       └── src/main.rs
└── docs/
    └── research/unreal_to_bevy/   # copy atlas here
```

---

## Phase 0 Verification Script

Run all before proceeding to Phase 1:

```bash
#!/bin/bash
set -e
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p aa_cli -- validate examples/demo_game
cargo run -p demo_game
```

| Check | Expected |
|-------|----------|
| Config loads | No panic; window opens |
| Prefab spawns | Entity in hierarchy |
| `validate` | Exit 0 |
| Schedules | No ambiguity errors at startup |

---

## What NOT to Build in Phase 0

| Temptation | Why wait |
|------------|----------|
| Networking | Phase 2 |
| GAS | Phase 1 |
| Editor UI | Phase 3 |
| World streaming | Phase 2 |
| Custom renderer | Phase 4 |

---

## Next Phase Entry

When Phase 0 checks pass, add crates in order:

1. `aa_tags`
2. `aa_input`
3. `aa_ability`
4. `aa_gameplay`
5. `aa_physics`
6. `aa_animation`
7. `aa_experience`

See `11_bevy_roadmap.md` Phase 1.

---

*This guide produces a compiling skeleton. Fill `todo!()` blocks incrementally with tests per crate.*
