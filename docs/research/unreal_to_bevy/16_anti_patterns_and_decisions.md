# 16 — Anti-Patterns and Architecture Decisions

> **Hard-won guardrails.** UE5 solved these problems over decades. Skip this doc and you will re-learn them painfully.

---

## Anti-Patterns (DO NOT)

### AP-01: Actor wrapper around ECS

```rust
// ❌ WRONG — fake OOP layer
struct Actor {
    id: u64,
    components: HashMap<TypeId, Box<dyn Any>>,
}

// ✅ RIGHT — pure ECS
commands.spawn((Pawn, Transform::default(), CharacterMovement::default()));
```

**Why:** Destroys SoA layout, blocks parallel queries, recreates UE problems without UE tooling.

---

### AP-02: Monolithic AbilitySystemComponent blob

```rust
// ❌ WRONG
struct AbilitySystemComponent {
    abilities: Vec<...>,
    effects: Vec<...>,
    attributes: HashMap<...>,
    tags: Vec<...>,
    // 2000 lines
}
```

**Why:** GAS is monolithic in UE because of UObject reflection. In ECS, split into components for parallel attribute/effect ticks.

**✅ Split:** `AbilityRegistry`, `ActiveEffects`, `AttributeSet`, `GameplayTags` (see `03_gameplay_framework.md`).

---

### AP-03: Putting player ASC on Pawn

**UE rule (from GAS README):** Players → ASC on **PlayerState**.

```rust
// ❌ WRONG for human players
commands.entity(pawn).insert(AbilityRegistry::default());

// ✅ RIGHT
commands.entity(player_state).insert(AbilityRegistry::default());
commands.entity(pawn).insert(AbilityUser { state: player_state });
```

**Why:** Pawn dies; abilities/cooldowns/tags must persist.

---

### AP-04: Direct attribute writes

```rust
// ❌ WRONG
health.current -= 10.0;

// ✅ RIGHT
commands.trigger(ApplyEffect { effect: "effects/damage_10", target, instigator });
```

**Why:** Breaks prediction, aggregation, kill credit chain.

---

### AP-05: Network-syncing everything

```rust
// ❌ WRONG
#[derive(Replicate)]
struct UiHoverState { ... }
```

**✅ Replicate only:** transforms (proxies), gameplay attributes, tags, ability grants, game state.

Use relevancy graph (see `08_networking.md`).

---

### AP-06: Spawning open world in one scene

**UE:** World Partition cells.

```rust
// ❌ WRONG — 100k entities in one Scene
Scene(entities: vec![...100000 items...])

// ✅ RIGHT — sector descriptors + streaming policy
```

---

### AP-07: Building Blueprint clone first

**UE:** Blueprint is 15+ years of editor investment.

**✅ Path:** RON data + Rust systems + AI agent codegen (`10_ai_native_game_studio.md`).

---

### AP-08: Editor as separate engine fork

```rust
// ❌ WRONG — duplicated game logic in editor crate
aa_editor::spawn_player() // different from aa_gameplay::spawn_player()

// ✅ RIGHT — shared plugins, SessionMode gate
```

---

### AP-09: Stringly-typed tags everywhere

```rust
// ❌ SLOW
if tags.contains("State.Stunned") { ... }

// ✅ FAST
if tags.has(TAG_STATE_STUNNED) { ... }  // TagId from dictionary
```

Load dictionary from `assets/data/tags.ron` at startup.

---

### AP-10: Ignoring fixed timestep for combat

Abilities + physics + movement must align on **FixedUpdate** (see `14_system_schedule_spec.md`).

---

## Architecture Decision Records (ADRs)

### ADR-001: ECS-native, not Actor-native

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Context** | UE Actor model is familiar but fights Bevy strengths |
| **Decision** | Map UE *responsibilities* to components; use Relationships for possession |
| **Consequences** | No Blueprint; need `aa_reflect` for editor |

---

### ADR-002: Text-first assets for agent workflow

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Context** | Cursor-like app requires diffable, validatable assets |
| **Decision** | RON for gameplay; TOML for config; binary only for mesh/audio |
| **Consequences** | Custom loaders; no `.uasset` compatibility |

---

### ADR-003: Rapier for physics backend

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Context** | Chaos is UE-internal; Rust needs mature solver |
| **Decision** | `bevy_rapier3d` behind `aa_physics` trait facade |
| **Consequences** | Vehicle/destruction may lag UE; acceptable for AA indie |

---

### ADR-004: lightyear/replicon for net (integrate not invent)

| Field | Value |
|-------|-------|
| **Status** | Proposed — verify at Phase 2 start |
| **Context** | Replication is multi-year effort |
| **Decision** | Integrate ecosystem crate; build relevancy graph on top |
| **Consequences** | API churn risk; pin versions |

---

### ADR-005: Compile-time Game Features before dynamic DLL

| Field | Value |
|-------|-------|
| **Status** | Accepted for MVP/AA v1 |
| **Context** | Lyra uses runtime GameFeature plugins; Rust dynamic loading is hard |
| **Decision** | Cargo feature crates first; `libloading` in AA v2 |
| **Consequences** | Rebuild to add mode; faster iteration early |

---

### ADR-006: egui editor MVP, not custom UI framework

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Context** | UnrealEd-scale UI is not feasible for small team |
| **Decision** | `bevy_egui` panels + agent chat external (Cursor) |
| **Consequences** | Less polish; acceptable for dev tool |

---

### ADR-007: Skip Lumen parity; staged GI

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Context** | Lumen = massive R&D |
| **Decision** | MVP: SSAO/baked; AA: probe DDGI; re-evaluate after ship |
| **Consequences** | Visual gap vs UE5 AAA; fine for stylized AA |

---

### ADR-008: Single App editor+play SessionMode

| Field | Value |
|-------|-------|
| **Status** | Accepted for MVP |
| **Context** | Dual App complicates viewport sync |
| **Decision** | One `App`, `SessionMode` resource toggles play/edit |
| **Consequences** | World reset on play; clone world in AA if needed |

---

## UE5 Lessons → Mandatory Rules

| UE lesson | Our rule |
|-----------|----------|
| Lyra ASC on PlayerState | Enforced in code review |
| GAS damage not TakeDamage | All damage via effects |
| Config hierarchy | Never hardcode ports/quality |
| World Partition for open world | Never single giant scene |
| Iris/Relevancy at scale | No broadcast replication |
| Init state machine | No gameplay until `InitStateCompleted` |
| Validate assets in CI | `aa_cli validate` gate on PR |

---

## Code Review Checklist

Copy into PR template:

- [ ] No `Actor` struct anti-pattern
- [ ] Ability changes go through effects
- [ ] ASC placement correct (PlayerState vs Pawn)
- [ ] New assets have `schema_version`
- [ ] Tags registered in dictionary
- [ ] Systems registered in `AaSchedule` order
- [ ] Server-only logic gated on `AppRole`
- [ ] `aa_cli validate` passes
- [ ] No `unwrap()` in gameplay systems (use `Result` + log)

---

## When to Break the Rules

| Situation | Allowed exception |
|-----------|-------------------|
| Prototype spike | Branch only; don't merge |
| Perf-critical inner loop | Profile first; document ADR |
| Third-party crate API | Thin adapter in `aa_*` |

---

*Review this document before every phase gate in `11_bevy_roadmap.md`.*
