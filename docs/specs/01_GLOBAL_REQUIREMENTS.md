# 01 — Global Requirements

> **Normative.** Cross-cutting requirements for all `aa_*` crates.

## Architecture

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-001 | The runtime MUST use Bevy ECS; systems MUST NOT wrap entities in class-based Actor containers | ADR-001 |
| REQ-GLOBAL-002 | Game code MUST depend on `aa_*` public APIs, not Bevy crate internals, except in `aa_*` adapter layers | Facade rule |
| REQ-GLOBAL-003 | All gameplay-tunable values MUST live in versioned text assets unless binary is required (mesh/audio) | ADR-002 |
| REQ-GLOBAL-004 | Every text asset MUST include `schema_version: u32` | Schema policy |
| REQ-GLOBAL-005 | Server-authoritative rules MUST NOT execute on pure clients | UE GameMode model |
| REQ-GLOBAL-006 | System ordering MUST conform to `14_system_schedule_spec.md` or successor SPEC schedule | Schedule law |

## Identity & Lifecycle

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-010 | Human player ability state MUST attach to PlayerState entity, NOT Pawn entity | GAS README |
| REQ-GLOBAL-011 | AI bot ability state MAY attach to Pawn entity | GAS README |
| REQ-GLOBAL-012 | Pawn possession MUST use bidirectional ECS Relationships (`Possesses`/`PossessedBy`) | Bevy 0.19 |
| REQ-GLOBAL-013 | Gameplay systems on pawns MUST query `Without<PendingInit>` until init FSM completes | Lyra init |
| REQ-GLOBAL-014 | Entity despawn MUST run cleanup systems for net IDs, abilities, and physics hooks | — |

## Gameplay Data

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-020 | Attribute modification MUST go through GameplayEffect application, never direct field writes in gameplay code | GAS README |
| REQ-GLOBAL-021 | All tags used in assets MUST be declared in project tag dictionary | GameplayTags |
| REQ-GLOBAL-022 | Kill credit / damage instigator chain MUST be preserved through effect application | GAS design goal |
| REQ-GLOBAL-023 | Cooldowns MUST be represented as gameplay tags OR explicit cooldown components, not ad-hoc timers in abilities | GAS |

## Networking (when `aa_net` enabled)

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-030 | Replicated components MUST be registered in `config/replication.toml` | Iris/Lyra |
| REQ-GLOBAL-031 | Server MUST own authoritative simulation for damage and attribute changes | — |
| REQ-GLOBAL-032 | Client cosmetic events (cues) MAY arrive via multicast; gameplay state MUST be server-driven | GAS replication |
| REQ-GLOBAL-033 | `NetEntityId` MUST remain stable for entity lifetime; MUST be recycled only after despawn confirmed | NetGUID analog |

## Assets & Validation

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-040 | `aa validate` MUST exit non-zero on any schema or reference error | Studio tier |
| REQ-GLOBAL-041 | Validator MUST detect cyclic prefab references | — |
| REQ-GLOBAL-042 | Validator MUST output SARIF 2.1 for agent consumption | CLI spec |
| REQ-GLOBAL-043 | Asset manifest MUST be regenerated on import; CI MUST check manifest freshness | AssetRegistry |

## Configuration

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-050 | Config MUST merge in order: engine_base → engine → platform → game → user → CLI | ConfigHierarchy.h |
| REQ-GLOBAL-051 | Scalability preset MUST apply atomically per frame boundary | Scalability.h |
| REQ-GLOBAL-052 | No hardcoded network ports, quality settings, or paths in gameplay systems | — |

## Tooling & Agent

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-060 | `AGENTS.md` MUST exist at project root | Studio |
| REQ-GLOBAL-061 | Every merged PR MUST pass `aa check` + `aa validate` | CI |
| REQ-GLOBAL-062 | Gameplay feature PRs MUST include or update playtest scenario | Studio |
| REQ-GLOBAL-063 | Agent-writeable paths MUST be allowlisted; `target/` and `.git/` MUST be blocked | Security |

## Error Handling

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-070 | Gameplay systems MUST NOT panic on bad designer data; MUST log error and skip | Bevy 0.16+ errors |
| REQ-GLOBAL-071 | Asset load failure MUST produce actionable error with asset ID and path | — |
| REQ-GLOBAL-072 | Network deserialization failure MUST disconnect client with reason code, not panic | — |

## Logging & Diagnostics

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-080 | All `aa_*` crates MUST use `tracing` with `target:` prefix | — |
| REQ-GLOBAL-081 | Playtest runs MUST capture structured log artifact | aa_cli |
| REQ-GLOBAL-082 | Net replication MUST support debug overlay for bytes/frame (dev builds) | — |

## Versioning

| ID | Requirement | Source |
|----|-------------|--------|
| REQ-GLOBAL-090 | `aa_engine` workspace MUST pin exact Bevy git tag or version | — |
| REQ-GLOBAL-091 | Schema breaking changes MUST increment `schema_version` with migrator | — |
| REQ-GLOBAL-092 | Public `aa_*` API breaking changes MUST follow semver | — |
