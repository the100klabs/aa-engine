# aa_world_stream - Subsystem Specification

> **Normative** | Priority P2 | UE ref: `WorldPartition/`

## Scope

### In scope

- Sector descriptors and streamed sector lifecycle
- World descriptors and region/sector hierarchy
- Runtime streaming sources and activation policy
- Data-layer filtering at sector activation
- Server-owned layer state in multiplayer
- Open-world inspection data for `aa world inspect`
- Deterministic sector cook metadata for post-AA open-world tracks

### Out of scope

- Terrain renderer implementation
- Navigation solver implementation
- Network transport implementation
- Editor viewport UI

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-STREAM-001 | Sector size MUST default 256m |
| REQ-STREAM-002 | Active window MUST default 5x5 around sources |
| REQ-STREAM-003 | Streaming policy MUST activate <= 2 sectors per frame by default |
| REQ-STREAM-004 | Async load p95 MUST be <= 500ms for AA target worlds |
| REQ-STREAM-005 | Data layers MUST filter spawn at activation |
| REQ-STREAM-006 | Server MUST own layer state in MP |
| REQ-STREAM-007 | Despawn on unload MUST not leave orphan `NetEntityId` |
| REQ-STREAM-008 | Gameplay systems MUST NOT require entities from inactive sectors |
| REQ-STREAM-009 | Sector descriptors MUST be text assets with `schema_version` and soft refs |
| REQ-STREAM-010 | Runtime MUST support multiple streaming sources |
| REQ-STREAM-011 | Activation priority MUST consider source distance, gameplay priority, layer state, and frame budget |
| REQ-STREAM-012 | Deactivation MUST run cleanup for entities, physics hooks, abilities, nav handles, audio emitters, and net IDs |
| REQ-STREAM-013 | Sector activation MUST be idempotent for identical source asset state |
| REQ-STREAM-014 | Sector unload MUST be reversible when no persistent world deltas apply |
| REQ-STREAM-015 | Persistent world deltas MUST be stored separately from authored sector baselines |
| REQ-STREAM-016 | Sector diagnostics MUST include entity count, memory estimate, load time, layer state, and dependency refs |
| REQ-STREAM-017 | `aa world inspect` data MUST be produced from the same registry used by runtime streaming and validate against `schemas/world_inspect_result.schema.json` |
| REQ-STREAM-018 | Worlds over 4 km2 MUST support hierarchy: region -> sector -> subcell |
| REQ-STREAM-019 | Sector cook MUST produce deterministic metadata for render, physics, nav, spawn, audio, and replication |
| REQ-STREAM-020 | Cook metadata MUST include source hashes and dependency hashes |
| REQ-STREAM-021 | Static distant environment density MUST support instancing or HLOD metadata, not one long-range entity per prop |
| REQ-STREAM-022 | AI spawn metadata MUST support simulation LOD hints: full, simplified, dormant, despawned |
| REQ-STREAM-023 | Network relevancy metadata MUST be sector-aware |
| REQ-STREAM-024 | Streaming profile artifacts MUST report p50/p95/p99 load latency and sector crossing hitches through `schemas/profile_summary_result.schema.json` |
| REQ-STREAM-025 | Validation MUST reject sector refs outside the project asset allowlist |
| REQ-STREAM-026 | Validation MUST reject cyclic region/sector/subcell dependencies |
| REQ-STREAM-027 | World descriptors MUST validate against `schemas/world.schema.json` |
| REQ-STREAM-028 | World descriptors MUST declare bounds, sector size, active window, data layers, regions, and sector refs |
| REQ-STREAM-029 | World sector refs MUST resolve to sector descriptors in the asset manifest |
| REQ-STREAM-030 | World descriptor data layers MUST be the only layers referenced by sectors in that world |
| REQ-STREAM-031 | `aa world inspect` MUST accept either a world asset path or world ID from `assets/worlds/` |
| REQ-STREAM-032 | Spawn table assets MUST validate against `schemas/spawn_table.schema.json` |
| REQ-STREAM-033 | AI profile assets referenced by spawn tables MUST validate against `schemas/ai_profile.schema.json` |
| REQ-STREAM-034 | Spawn table entries MUST resolve pawn refs, optional prefab refs, and optional AI profile refs before sector activation |
| REQ-STREAM-035 | Spawn metadata cook MUST include spawn table hashes and AI profile hashes for deterministic sector activation |
| REQ-STREAM-036 | `aa world cook --json` output MUST validate against `schemas/world_cook_result.schema.json` |

## API Contract

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorId {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionId {
    pub x: i32,
    pub y: i32,
}

pub struct StreamingSource {
    pub id: String,
    pub position_m: Vec3,
    pub radius_sectors: u32,
    pub priority: u8,
}

pub struct SectorDescriptor {
    pub schema_version: u32,
    pub id: SectorId,
    pub region: RegionId,
    pub bounds_m: Aabb,
    pub layers: Vec<String>,
    pub refs: Vec<String>,
    pub cook: SectorCookMetadata,
}

pub struct WorldDescriptor {
    pub schema_version: u32,
    pub id: String,
    pub bounds_m: Aabb,
    pub sector_size_m: f32,
    pub active_window: UVec2,
    pub data_layers: Vec<WorldDataLayer>,
    pub regions: Vec<RegionDescriptor>,
}

pub struct WorldDataLayer {
    pub id: String,
    pub default_state: DataLayerState,
    pub server_authoritative: bool,
}

pub enum DataLayerState {
    Active,
    Loaded,
    Unloaded,
}

pub struct RegionDescriptor {
    pub id: String,
    pub coord: IVec2,
    pub bounds_m: Aabb,
    pub sectors: Vec<SectorRef>,
}

pub struct SectorRef {
    pub id: String,
    pub coord: IVec2,
    pub path: String,
    pub required_layers: Vec<String>,
}

pub struct SectorCookMetadata {
    pub source_hash: String,
    pub render_hash: Option<String>,
    pub physics_hash: Option<String>,
    pub nav_hash: Option<String>,
    pub spawn_hash: Option<String>,
    pub audio_hash: Option<String>,
    pub replication_hash: Option<String>,
}

pub struct SectorDiagnostics {
    pub id: SectorId,
    pub loaded: bool,
    pub active: bool,
    pub entity_count: u32,
    pub memory_estimate_mb: f32,
    pub load_ms: Option<f32>,
    pub refs: Vec<String>,
    pub layers: Vec<String>,
}
```

## Streaming Lifecycle

```text
discovered -> queued -> loading -> loaded -> activating -> active
active -> deactivating -> loaded -> unloading -> discovered
```

| Transition | Required behavior |
|------------|-------------------|
| discovered -> queued | policy selects sector within active window or priority request |
| queued -> loading | async IO starts, no gameplay entities spawned |
| loading -> loaded | source assets and cook metadata available |
| loaded -> activating | layer filter and budget check pass |
| activating -> active | entities spawned, diagnostics updated, events emitted |
| active -> deactivating | gameplay cleanup and delta capture run |
| deactivating -> loaded | no live gameplay state remains |
| loaded -> unloading | asset handles released when budget requires |

## Data Invariants

- A sector descriptor MUST NOT contain absolute filesystem paths.
- A world descriptor MUST NOT contain absolute filesystem paths.
- A world descriptor MUST NOT reference sectors outside the project allowlist.
- A sector MUST NOT spawn gameplay entities when its required data layer is disabled.
- A sector unload MUST NOT despawn `PlayerState` entities.
- Server layer state MUST override client layer guesses in multiplayer.
- Cook metadata MUST be stale when any source hash changes.
- Persistent deltas MUST reference stable authored IDs, not transient ECS entity IDs.

## CLI Integration

`aa_world_stream` MUST provide the data needed by:

- `aa world inspect --world <id|path>`
- `aa world inspect --world <id|path> --live`
- `aa world generate --template <name>`
- `aa world cook --world <id|path> --verify`
- `aa profile summarize <artifact_path>`

The CLI owns command parsing. `aa_world_stream` owns sector registry, validation, cook metadata, live diagnostics, and the data models behind:

- `docs/specs/schemas/world_inspect_result.schema.json`
- `docs/specs/schemas/world_cook_result.schema.json`
- `docs/specs/schemas/profile_summary_result.schema.json`

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-STREAM-01 | Walk across 3 sectors | 0 crash | playtest |
| T-STREAM-02 | Layer toggle | entities appear/hide | integration |
| T-STREAM-03 | Load perf | p95 <= 500ms | bench |
| T-STREAM-04 | Multiple streaming sources | union of active windows loads within budget | integration |
| T-STREAM-05 | Sector unload cleanup | no orphan entity, physics, ability, nav, audio, or net state | integration |
| T-STREAM-06 | Persistent delta reload | authored baseline plus delta restores expected state | playtest |
| T-STREAM-07 | Inspect registry | `aa world inspect` fixture matches sector registry | integration |
| T-STREAM-08 | Cook determinism | unchanged inputs produce identical metadata hashes | integration |
| T-STREAM-09 | Dependency cycle | validator rejects cyclic region/sector/subcell refs | integration |
| T-STREAM-10 | Relevancy metadata | distant sector entities excluded from net interest set | net integration |
| T-STREAM-11 | Open-world traversal profile | sector crossing hitch within current gate budget | playtest/profile |
| T-STREAM-12 | World descriptor schema | malformed world descriptor rejected | unit |
| T-STREAM-13 | World sector refs | missing sector ref returns `REF_MISSING` | integration |
| T-STREAM-14 | World layer refs | undeclared sector layer rejected | integration |
| T-STREAM-15 | Spawn table schema | malformed spawn table rejected | unit |
| T-STREAM-16 | AI profile schema | malformed AI profile rejected | unit |
| T-STREAM-17 | Spawn refs | missing pawn/profile/prefab returns `REF_MISSING` | integration |
| T-STREAM-18 | Cook result schema | `aa world cook --json` validates schema | unit |
| T-STREAM-19 | Profile summary schema | `aa profile summarize --json` validates schema | unit |

## Acceptance

**P2 certified when:** T-STREAM-01-03 green.

**Open World Alpha certified when:** T-STREAM-01-09 and T-STREAM-12-17 green + Gate OWA pass in `04_ACCEPTANCE_GATES.md`.

**Open World Beta certified when:** T-STREAM-01-19 green + Gate OWB pass in `04_ACCEPTANCE_GATES.md`.
