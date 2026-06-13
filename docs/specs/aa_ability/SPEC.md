# aa_ability — Subsystem Specification

> **Normative** | Priority P1 | Depends: `aa_tags`, `aa_core`, `aa_net` (optional)

## Scope

### In scope
- Gameplay Ability System (GAS) subset per UE5 `GameplayAbilities` plugin
- Split ECS components: `AbilityRegistry`, `ActiveEffects`, `AttributeSet`, `GameplayTags`
- GameplayEffect application + modifier aggregation
- GameplayCue events (decoupled FX)
- Cooldowns via tags
- Server-authoritative activation; client prediction hooks for AA

### Out of scope
- Full Blueprint ability tasks
- WASM ability scripts (post-AA)
- UI damage numbers

## UE5 Reference
- `Engine/Plugins/Runtime/GameplayAbilities/README.md`
- `AbilitySystemComponent.h`, `GameplayEffect.h`, `AttributeSet.h`
- `GameplayPrediction.h`

---

## Requirements

### ASC & Placement

| ID | Requirement |
|----|-------------|
| REQ-ABILITY-001 | Human player `AbilityRegistry` MUST be on PlayerState entity |
| REQ-ABILITY-002 | AI bot `AbilityRegistry` MAY be on Pawn entity |
| REQ-ABILITY-003 | Pawn MUST reference ASC via `AbilityUser { state: Entity }` component |
| REQ-ABILITY-004 | `AbilityRegistry` MUST NOT be a single monolithic struct > 1KB inline |

### Abilities

| ID | Requirement |
|----|-------------|
| REQ-ABILITY-010 | `try_activate(AbilityId, &ActivationContext)` MUST check tags, cooldowns, costs before activate |
| REQ-ABILITY-011 | Failed activation MUST return `AbilityError` without side effects |
| REQ-ABILITY-012 | Active ability MUST emit `AbilityActivatedEvent` |
| REQ-ABILITY-013 | Ability MUST load definition from `GameplayAbility` RON asset |
| REQ-ABILITY-014 | Rust `AbilityImpl` registrar MUST map `impl` string from asset to handler |

### Effects & Attributes

| ID | Requirement |
|----|-------------|
| REQ-ABILITY-020 | Attributes MUST have `base` and `current` values |
| REQ-ABILITY-021 | All `current` modifications MUST go through `apply_effect(EffectSpec)` |
| REQ-ABILITY-022 | Modifier aggregation MUST add `Multiply` modifiers before apply (UE GAS rule) |
| REQ-ABILITY-023 | `Health` MUST clamp to `[min, max]` after aggregation |
| REQ-ABILITY-024 | Periodic effects MUST tick on `FixedUpdate` at `period` interval |
| REQ-ABILITY-025 | Effect application MUST record `instigator` and `causer` entities |
| REQ-ABILITY-026 | Instant effects MUST apply same frame; duration effects MUST have `remaining` |
| REQ-ABILITY-027 | Attribute set assets MUST validate against `schemas/attribute_set.schema.json` |
| REQ-ABILITY-028 | Attribute set validation MUST enforce unique attribute names and `min <= default <= max` |
| REQ-ABILITY-029 | GameplayEffect modifier attributes MUST resolve to an attribute granted by the target ASC before application |

### Tags

| ID | Requirement |
|----|-------------|
| REQ-ABILITY-030 | Effects MUST grant/remove tags per asset definition |
| REQ-ABILITY-031 | Ability blocking tags MUST prevent activation (REQ-ABILITY-010) |

### Cues

| ID | Requirement |
|----|-------------|
| REQ-ABILITY-040 | `GameplayCueEvent` MUST be emitted on cue tag; MUST NOT modify gameplay state |
| REQ-ABILITY-041 | Cue listeners MUST be decoupled (VFX/audio systems subscribe) |

### Replication & Prediction (AA)

| ID | Requirement |
|----|-------------|
| REQ-ABILITY-060 | Server MUST be authoritative for effect application |
| REQ-ABILITY-061 | Owning client MAY predict instant effects; MUST rollback on server reject |
| REQ-ABILITY-062 | Predicted activation MUST complete within 3 frames or cancel |
| REQ-ABILITY-063 | Simulated proxies MUST NOT run ability activation logic |

### Performance

| ID | Requirement |
|----|-------------|
| REQ-ABILITY-070 | 100 ASCs + 50 effects MUST tick ≤ 0.5ms (see `02_PERFORMANCE_BUDGETS.md`) |

---

## API Contract

```rust
pub type AbilityId = AssetPath;  // "abilities/fireball"
pub type EffectId = AssetPath;
pub type AttributeId = InternedString;
pub type TagId = crate::aa_tags::TagId;

#[derive(Component, Default)]
pub struct AbilityRegistry {
    pub granted: Vec<GrantedAbility>,
}

#[derive(Component)]
pub struct AbilityUser {
    pub state: Entity,  // PlayerState or self for AI
}

#[derive(Component, Default)]
pub struct ActiveEffects {
    pub instances: Vec<ActiveEffectInstance>,
}

#[derive(Component)]
pub struct AttributeSet {
    pub attributes: Vec<AttributeValue>,
}

#[derive(Clone)]
pub struct AttributeValue {
    pub id: AttributeId,
    pub base: f32,
    pub current: f32,
    pub min: f32,
    pub max: f32,
}

pub struct ActivationContext<'a> {
    pub world: &'a World,
    pub instigator: Entity,
    pub asc: Entity,
}

#[derive(Debug, thiserror::Error)]
pub enum AbilityError {
    #[error("on cooldown")]
    OnCooldown,
    #[error("blocked by tag {0:?}")]
    BlockedByTag(TagId),
    #[error("missing tag {0:?}")]
    MissingTag(TagId),
    #[error("insufficient resource")]
    InsufficientCost,
    #[error("already active")]
    AlreadyActive,
}

pub trait AbilityImpl: Send + Sync {
    fn activate(&self, ctx: &mut ActivationContext) -> Result<(), AbilityError>;
    fn cancel(&self, ctx: &mut ActivationContext);
}

impl AbilityRegistry {
    pub fn try_activate(
        &mut self,
        id: AbilityId,
        ctx: &mut ActivationContext,
        impls: &AbilityImplRegistry,
    ) -> Result<ActivationHandle, AbilityError>;
}

pub fn apply_effect(
    world: &mut World,
    spec: &EffectSpec,
    target_asc: Entity,
    instigator: Entity,
) -> Result<EffectHandle, EffectApplyError>;

#[derive(Event)]
pub struct GameplayCueEvent {
    pub cue_tag: TagId,
    pub target: Entity,
    pub instigator: Entity,
    pub location: Option<Vec3>,
}

#[derive(Event)]
pub struct AbilityActivatedEvent {
    pub ability: AbilityId,
    pub asc: Entity,
}
```

---

## Data Schema

Formal: `docs/specs/schemas/gameplay_ability.schema.json`, `gameplay_effect.schema.json`, `attribute_set.schema.json`

---

## Invariants

1. `current` attribute only changes inside `apply_effect` aggregation
2. Dead targets (`State.Dead` tag) reject new effects unless asset overrides
3. Cue events never write to `AttributeSet`
4. `AbilityRegistry` on despawned PlayerState is invalid — cleanup MUST run

---

## Test Matrix

| ID | Scenario | Input | Expected | Auto |
|----|----------|-------|----------|------|
| T-ABILITY-01 | Fireball damage | apply burning effect | Health −5/sec × 5 | integration |
| T-ABILITY-02 | Multiply agg | +10% and +30% damage | +40% total | unit |
| T-ABILITY-03 | Stun block | activate while stunned | `BlockedByTag` | unit |
| T-ABILITY-04 | ASC survives death | despawn pawn, respawn | abilities persist on PlayerState | integration |
| T-ABILITY-05 | Instigator chain | A applies effect to B via C | kill credit A | unit |
| T-ABILITY-06 | Cue decoupled | apply effect | cue event, no attribute change from listener | unit |
| T-ABILITY-07 | Predict rollback | client predict, server reject | attribute restored | integration |
| T-ABILITY-08 | Perf 100 ASC | bench | ≤ 0.5ms | bench |
| T-ABILITY-09 | Attribute schema | malformed attribute set rejected | unit |
| T-ABILITY-10 | Attribute bounds | default outside min/max rejected | unit |
| T-ABILITY-11 | Unknown modifier attr | effect targets missing attribute | validation error | integration |

---

## Acceptance

**P1 certified when:** T-ABILITY-01–06 and T-ABILITY-09–11 green + playtest `fireball_hit` PASS (Gate P1-02).

**AA certified when:** T-ABILITY-07–08 green + REQ-ABILITY-060–063 on dedicated server test.
