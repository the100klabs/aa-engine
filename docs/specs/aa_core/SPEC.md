# aa_core — Subsystem Specification

> **Normative** | Priority P0 | Depends: Bevy `bevy_app`, `bevy_ecs`

## Scope

### In scope
- App boot, plugin registration order enforcement
- Config layer merge (TOML)
- Console variables (CVars)
- `AppRole` (client/server/editor)
- `AaSchedule` system set labels
- Diagnostics (`tracing` setup)

### Out of scope
- Gameplay logic, assets, networking

## UE5 Reference
- `Engine/Source/Runtime/Core/Public/Misc/ConfigHierarchy.h`
- `Engine/Config/BaseEngine.ini`
- `UnrealEditor.Target.cs` / `UnrealGame.Target.cs`

---

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-CORE-001 | `AaCorePlugin` MUST register before all other `aa_*` plugins |
| REQ-CORE-002 | Config merge MUST implement layers in REQ-GLOBAL-050 exactly |
| REQ-CORE-003 | `ConfigProvider::get("a.b.c")` MUST return merged value or documented default |
| REQ-CORE-004 | CLI `--set key=value` MUST override user layer at runtime |
| REQ-CORE-005 | `AppRole` MUST be set at startup and MUST NOT change at runtime |
| REQ-CORE-006 | `AaSchedule` sets MUST be registered in `Schedule` with explicit ordering |
| REQ-CORE-007 | CVars MUST be readable from config and overridable at runtime in dev builds |
| REQ-CORE-008 | `aa_core` MUST initialize `tracing` with env filter `AA_LOG` |
| REQ-CORE-009 | Plugin order violation MUST panic in dev, log error in release |
| REQ-CORE-010 | `engine_base.toml` MUST ship inside `aa_core` crate |
| REQ-CORE-011 | `config/engine.toml` MUST validate against `schemas/config_engine.schema.json` when present |
| REQ-CORE-012 | `config/game.toml` MUST validate against `schemas/config_game.schema.json` when present |
| REQ-CORE-013 | `config/input.toml` MUST validate against `schemas/config_input.schema.json` when present |
| REQ-CORE-014 | `config/scalability.toml` MUST validate against `schemas/config_scalability.schema.json` when present |
| REQ-CORE-015 | Missing optional config files MUST use documented defaults, but malformed present files MUST fail validation |

---

## API Contract

```rust
/// Application role — set once at boot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppRole {
    Client,
    DedicatedServer,
    ListenServer,
    Editor,
}

/// Merged configuration — thread-safe read.
pub struct ConfigProvider {
    // private
}

impl ConfigProvider {
    /// Load from project root. MUST implement layer merge REQ-GLOBAL-050.
    pub fn load(project_root: &std::path::Path) -> Result<Self, ConfigError>;

    /// Dot-path lookup. Returns None if missing (no silent default unless documented).
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T>;

    /// Apply CLI override to highest-priority layer.
    pub fn apply_cli_override(&mut self, key: &str, value: &str) -> Result<(), ConfigError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required config key: {0}")]
    MissingRequired(&'static str),
    #[error("parse error at {key}: {source}")]
    Parse { key: String, source: Box<dyn std::error::Error + Send + Sync> },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Runtime console variable.
pub struct CVar<T> {
    // get/set with change events
}

pub struct AaCorePlugin {
    pub role: AppRole,
}

impl Plugin for AaCorePlugin {
    fn build(&self, app: &mut App);
}

/// System set labels — all aa_* plugins configure into these.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AaSchedule {
    FrameStart,
    Input,
    NetReceive,
    AbilityInput,
    MovementIntent,
    AbilityFixed,
    Physics,
    Effects,
    Crowd,
    RootMotion,
    InitState,
    Animation,
    GameplayCues,
    WorldStream,
    Camera,
    WorldStreamApply,
    NetSend,
    Interpolation,
    FrameEnd,
}
```

---

## Invariants

1. `ConfigProvider` is immutable after `Startup` except dev CVars
2. `AaSchedule` ordering is total and acyclic
3. `DedicatedServer` role MUST NOT add render plugins via `aa_core`

---

## Performance

| Metric | Budget |
|--------|--------|
| Config load | ≤ 50ms |
| CVar apply | ≤ 0.01ms |

---

## Test Matrix

| ID | Scenario | Input | Expected | Auto |
|----|----------|-------|----------|------|
| T-CORE-01 | Config merge order | project overrides engine | project value wins | unit |
| T-CORE-02 | CLI override | `--set net.port=9999` | get returns 9999 | unit |
| T-CORE-03 | Plugin order | register `aa_ability` before `aa_core` | dev panic | integration |
| T-CORE-04 | Server no render | `AppRole::DedicatedServer` | no window plugin | integration |
| T-CORE-05 | Schedule acyclic | all plugins registered | 0 ambiguities | integration |
| T-CORE-06 | Config schema valid | baseline `config/*.toml` | schema pass | integration |
| T-CORE-07 | Malformed config | invalid `net.default_port` | validation error | unit |

---

## Acceptance

**P0 certified when:** T-CORE-01 through T-CORE-07 green + REQ-CORE-001–015 implemented.
