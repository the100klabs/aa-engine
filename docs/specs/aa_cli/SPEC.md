# aa_cli — Subsystem Specification

> **Normative** | Priority P0 | **Studio wedge — build first**

## Scope

### In scope
- CLI commands for check, validate, playtest, index, scene, run
- Post-AA world commands for inspect, generate, and cook verification
- Agent evaluation commands for prompt-to-feature suites
- JSON stdout for agents; human text stderr
- SARIF 2.1 validation output
- Exit code contract

### Out of scope
- GUI chat UI (external Cursor-like app)
- Cloud CI orchestration

## UE5 Reference
- `AutomationTool`, `DataValidation` plugin, commandlets

---

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-CLI-001 | `aa check` MUST run `cargo check` on workspace and return structured diagnostics JSON |
| REQ-CLI-002 | `aa validate` MUST validate all RON/TOML assets against JSON schemas |
| REQ-CLI-003 | `aa validate` MUST verify all soft references resolve in asset manifest |
| REQ-CLI-004 | `aa validate` MUST detect cyclic prefab refs |
| REQ-CLI-005 | `aa validate --format sarif` MUST emit SARIF 2.1 |
| REQ-CLI-006 | `aa validate` on 1000 assets MUST complete ≤ 10s (AA Target HW) |
| REQ-CLI-007 | `aa playtest` MUST run scenario RON and return pass/fail JSON |
| REQ-CLI-008 | `aa playtest` MUST write log artifact path in output |
| REQ-CLI-009 | `aa index --query` MUST return ranked hits JSON |
| REQ-CLI-010 | `aa scene patch` MUST support `--dry-run` and return `undo_token` |
| REQ-CLI-011 | Exit codes MUST match `17_agent_cli_contract.md` / this spec |
| REQ-CLI-012 | stdout MUST be JSON when `--json` flag set (default for agents) |
| REQ-CLI-013 | `aa run --role` MUST support `client`, `dedicated_server`, `editor` |
| REQ-CLI-014 | Commands MUST NOT write outside project allowlist |
| REQ-CLI-015 | `aa ability graph <id>` MUST return dependency nodes/edges JSON |
| REQ-CLI-016 | `aa world inspect` MUST return sector bounds, layers, refs, cooked artifact status, and budget summaries JSON |
| REQ-CLI-017 | `aa world inspect --live` MUST include active sector state when connected to a running editor/game session |
| REQ-CLI-018 | `aa world generate` MUST create region/sector starter assets only from versioned templates |
| REQ-CLI-019 | `aa world generate --dry-run` MUST return the files and assets it would create without writing them |
| REQ-CLI-020 | `aa world cook --verify` MUST verify render, physics, nav, spawn, audio, and replication metadata artifacts |
| REQ-CLI-021 | `aa world cook` MUST be deterministic for unchanged source assets and config |
| REQ-CLI-022 | `aa eval run` MUST execute versioned prompt-to-feature evals and return machine-readable reports |
| REQ-CLI-023 | `aa eval run` reports MUST include task id, pass/fail, repair attempts, commands run, artifacts, and final diff path |
| REQ-CLI-024 | `aa profile summarize` MUST return frame, sector, IO, memory, replication, and hitch summaries from captured artifacts |
| REQ-CLI-025 | `aa validate` MUST validate `assets/playtests/*.ron` against `schemas/playtest_scenario.schema.json` |
| REQ-CLI-026 | `aa validate` MUST validate `assets/evals/*.ron` against `schemas/agent_eval.schema.json` |
| REQ-CLI-027 | `aa eval run` MUST reject eval tasks whose required commands or acceptance criteria are not declared in the eval asset |
| REQ-CLI-028 | `aa scene patch --patch <file>` MUST validate patch files against `schemas/scene_patch.schema.json` before applying or dry-running |
| REQ-CLI-029 | `aa scene patch --dry-run` MUST report affected entity IDs, affected files, validation errors, and an undo token without writing files |
| REQ-CLI-030 | `aa validate` MUST validate `assets/asset_manifest.json` against `schemas/asset_manifest.schema.json` when present |
| REQ-CLI-031 | `aa validate` MUST validate `assets/worlds/*.ron` against `schemas/world.schema.json` |
| REQ-CLI-032 | `aa world inspect --world <id|path>` MUST load the world descriptor, resolve sector refs, and report missing refs as validation errors |
| REQ-CLI-033 | `aa validate` MUST validate `assets/pawns/*.ron` against `schemas/pawn_data.schema.json` |
| REQ-CLI-034 | `aa validate` MUST validate `assets/input/contexts/*.ron` against `schemas/input_context.schema.json` |
| REQ-CLI-035 | `aa validate` MUST validate `assets/action_sets/*.ron` against `schemas/action_set.schema.json` |
| REQ-CLI-036 | `aa validate` MUST validate `assets/attributes/*.ron` against `schemas/attribute_set.schema.json` |
| REQ-CLI-037 | `aa validate` MUST report unknown GameplayEffect modifier attributes when no reachable AttributeSet declares them |
| REQ-CLI-038 | `aa validate` MUST validate `assets/ai/*.ron` against `schemas/ai_profile.schema.json` |
| REQ-CLI-039 | `aa validate` MUST validate `assets/spawn_tables/*.ron` against `schemas/spawn_table.schema.json` |
| REQ-CLI-040 | `aa validate` MUST resolve spawn table pawn, prefab, and AI profile refs before playtest |
| REQ-CLI-041 | `aa index --query` output MUST validate against `schemas/index_result.schema.json` |
| REQ-CLI-042 | `aa index --query` hits MUST include score, kind, path, summary, and stale status |
| REQ-CLI-043 | `aa index --query` MUST include spec, schema, asset, playtest, and eval hits when relevant to the query |
| REQ-CLI-044 | `aa validate --format json` output MUST validate against `schemas/validation_result.schema.json` |
| REQ-CLI-045 | `aa check --json` output MUST validate against `schemas/check_result.schema.json` |
| REQ-CLI-046 | Check and validation diagnostics MUST use project-relative paths and include line/column spans when available |
| REQ-CLI-047 | `aa playtest --json` output MUST validate against `schemas/playtest_result.schema.json` |
| REQ-CLI-048 | `aa eval run --json` output MUST validate against `schemas/eval_report.schema.json` |
| REQ-CLI-049 | Playtest and eval report artifacts MUST use project-relative paths |
| REQ-CLI-050 | `aa world inspect --json` output MUST validate against `schemas/world_inspect_result.schema.json` |
| REQ-CLI-051 | `aa world cook --json` output MUST validate against `schemas/world_cook_result.schema.json` |
| REQ-CLI-052 | `aa profile summarize --json` output MUST validate against `schemas/profile_summary_result.schema.json` |

---

## API Contract (CLI surface)

```
aa new <name> [--template <t>]
aa check [path] [--json]
aa validate [path] [--format sarif|json]
aa playtest --scenario <id> [--duration <dur>] [--headless] [--json]
aa index [path] [--query <q>] [--json]
aa scene list --scene <path> [--filter <f>] [--json]
aa scene inspect <entity_id> --scene <path> [--json]
aa scene patch --scene <path> --patch <file> [--dry-run] [--json]
aa run --project <path> --role <client|dedicated_server|editor>
aa config get <key> --project <path> [--json]
aa ability graph <ability_id> [--json]
aa world inspect --world <id|path> [--live] [--json]
aa world generate --template <name> --output <path> [--name <id>] [--dry-run] [--json]
aa world cook --world <id|path> [--verify] [--json]
aa eval list [--json]
aa eval run <eval_id|suite> [--max-repairs <n>] [--json]
aa profile summarize <artifact_path> [--json]
aa --version [--json]
```

### Scene patch schema

**Patch file:** JSON matching `docs/specs/schemas/scene_patch.schema.json`

Patch operations MUST address stable `SceneEntityId` values, never transient ECS `Entity` IDs. Implementations MUST reject absolute paths, parent-directory paths, and writes outside the project allowlist before any mutation.

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation failed |
| 2 | Compile failed |
| 3 | Playtest failed |
| 4 | Invalid arguments |
| 5 | Internal error |

### Validate output schema (JSON)

**Formal schema:** `docs/specs/schemas/validation_result.schema.json`

```json
{
  "ok": true,
  "error_count": 0,
  "warning_count": 0,
  "diagnostics": [
    {
      "code": "REF_MISSING",
      "severity": "error",
      "message": "string",
      "path": "assets/abilities/fireball.ron",
      "span": { "line": 12, "column": 4 }
    }
  ],
  "duration_ms": 842
}
```

### Check output schema

**Formal schema:** `docs/specs/schemas/check_result.schema.json`

```json
{
  "ok": false,
  "error_count": 1,
  "warning_count": 0,
  "duration_ms": 2300,
  "command": "cargo check --workspace",
  "diagnostics": [
    {
      "code": "E0432",
      "severity": "error",
      "message": "unresolved import",
      "package": "demo_game",
      "spans": [
        {
          "path": "src/main.rs",
          "line_start": 3,
          "column_start": 5,
          "is_primary": true
        }
      ]
    }
  ]
}
```

### Index output schema

**Formal schema:** `docs/specs/schemas/index_result.schema.json`

```json
{
  "ok": true,
  "query": "enemy camp sector",
  "duration_ms": 42,
  "hits": [
    {
      "id": "REQ-AAA-STUDIO-003",
      "kind": "spec",
      "path": "docs/specs/06_AAA_OPEN_WORLD_STUDIO.md",
      "score": 0.97,
      "summary": "aa world inspect must report sector bounds, layers, refs, cook artifacts, budgets, and live activation state.",
      "span": { "line_start": 72 },
      "relations": [{ "kind": "requires", "target": "aa world inspect" }],
      "stale": false
    }
  ]
}
```

### Playtest output schema

**Formal schema:** `docs/specs/schemas/playtest_result.schema.json`

```json
{
  "ok": true,
  "scenario": "fireball_hit",
  "duration_secs": 30,
  "assertions": [
    { "name": "target_health_delta", "passed": true, "expected": -25, "actual": -25 }
  ],
  "artifacts": { "log": "artifacts/playtests/fireball_hit.log", "trace": "artifacts/playtests/fireball_hit.trace" }
}
```

### Scene patch output schema

**Formal schema:** `docs/specs/schemas/scene_patch_result.schema.json`

```json
{
  "ok": true,
  "dry_run": true,
  "patch_id": "add_campfire_preview",
  "target": "assets/sectors/sector_0_0.ron",
  "affected_files": [
    "assets/sectors/sector_0_0.ron",
    "assets/prefabs/camp_fire.ron"
  ],
  "affected_entities": ["sector_0_0/camp_fire_preview"],
  "ops": [
    {
      "index": 0,
      "kind": "InstantiatePrefab",
      "entity_id": "sector_0_0/camp_fire_preview",
      "affected_files": ["assets/prefabs/camp_fire.ron"]
    }
  ],
  "undo_token": "undo:dry-run:892d4bdd3226a3ce",
  "diagnostics": [],
  "duration_ms": 2
}
```

### Scene list output schema

**Formal schema:** `docs/specs/schemas/scene_list_result.schema.json`

```json
{
  "ok": true,
  "scene": "assets/sectors/sector_0_0.ron",
  "kind": "sector",
  "entity_count": 2,
  "entities": [
    {
      "id": "sector_0_0/entity_0",
      "name": "camp_fire",
      "prefab": "assets/prefabs/camp_fire.ron",
      "layers": ["terrain", "gameplay", "encounters"]
    }
  ],
  "diagnostics": [],
  "duration_ms": 2
}
```

### Scene inspect output schema

**Formal schema:** `docs/specs/schemas/scene_inspect_result.schema.json`

```json
{
  "ok": true,
  "scene": "assets/sectors/sector_0_0.ron",
  "kind": "sector",
  "entity_id": "sector_0_0/entity_0",
  "entity": {
    "id": "sector_0_0/entity_0",
    "name": "camp_fire",
    "prefab": "assets/prefabs/camp_fire.ron",
    "transform": { "translation": [32.0, 0.0, -18.0], "rotation_y_degrees": 0.0, "scale": [1.0, 1.0, 1.0] }
  },
  "diagnostics": [],
  "duration_ms": 2
}
```

### Ability graph output schema

**Formal schema:** `docs/specs/schemas/ability_graph_result.schema.json`

```json
{
  "ok": true,
  "ability_id": "basic_melee",
  "ability_asset": "assets/abilities/basic_melee.ron",
  "nodes": [
    {
      "id": "ability:basic_melee",
      "kind": "ability",
      "label": "basic_melee",
      "path": "assets/abilities/basic_melee.ron"
    },
    {
      "id": "tag:State.Stunned",
      "kind": "tag",
      "label": "State.Stunned"
    }
  ],
  "edges": [
    {
      "from": "ability:basic_melee",
      "to": "tag:State.Stunned",
      "kind": "blocked_by_tag"
    }
  ],
  "diagnostics": [],
  "duration_ms": 4
}
```

### World inspect output schema

**Formal schema:** `docs/specs/schemas/world_inspect_result.schema.json`

```json
{
  "ok": true,
  "world": "open_world_studio",
  "world_asset": "assets/worlds/open_world_studio.ron",
  "bounds_m": { "min": [-2048, 0, -2048], "max": [2048, 1024, 2048] },
  "sector_count": 64,
  "layers": ["base", "gameplay", "foliage"],
  "sectors": [
    {
      "id": "sector_0_0",
      "path": "assets/sectors/sector_0_0.ron",
      "coord": [0, 0],
      "bounds_m": { "min": [0, 0, 0], "max": [256, 512, 256] },
      "refs": ["assets/sectors/sector_0_0.ron"],
      "layers": ["base", "gameplay"],
      "cooked": {
        "render": { "present": true, "stale": false, "hash": "sha256:abc", "artifact": "artifacts/cook/sector_0_0.render" },
        "physics": { "present": true, "stale": false, "hash": "sha256:def", "artifact": "artifacts/cook/sector_0_0.physics" },
        "nav": { "present": true, "stale": false, "hash": "sha256:ghi", "artifact": "artifacts/cook/sector_0_0.nav" },
        "spawn": { "present": true, "stale": false, "hash": "sha256:jkl", "artifact": "artifacts/cook/sector_0_0.spawn" },
        "audio": { "present": true, "stale": false, "hash": "sha256:mno", "artifact": "artifacts/cook/sector_0_0.audio" },
        "replication": { "present": true, "stale": false, "hash": "sha256:pqr", "artifact": "artifacts/cook/sector_0_0.replication" }
      },
      "budgets": { "entities": 1200, "authored_objects": 3200, "memory_mb": 48, "load_p95_ms": 220 }
    }
  ],
  "budgets": { "authored_objects": 250000, "visible_instanced_props": 100000, "memory_mb": 4096, "load_p95_ms": 390 },
  "live": {
    "connected": true,
    "active_sectors": ["sector_0_0"],
    "streaming_sources": ["player_0"]
  },
  "duration_ms": 38
}
```

### World cook output schema

**Formal schema:** `docs/specs/schemas/world_cook_result.schema.json`

```json
{
  "ok": true,
  "world": "open_world_studio",
  "world_asset": "assets/worlds/open_world_studio.ron",
  "verified": true,
  "deterministic": true,
  "sector_count": 64,
  "source_hash": "sha256:world",
  "artifacts": [
    {
      "sector": "sector_0_0",
      "source_hash": "sha256:source",
      "render": { "path": "artifacts/cook/sector_0_0.render", "hash": "sha256:abc", "verified": true },
      "physics": { "path": "artifacts/cook/sector_0_0.physics", "hash": "sha256:def", "verified": true },
      "nav": { "path": "artifacts/cook/sector_0_0.nav", "hash": "sha256:ghi", "verified": true },
      "spawn": { "path": "artifacts/cook/sector_0_0.spawn", "hash": "sha256:jkl", "verified": true },
      "audio": { "path": "artifacts/cook/sector_0_0.audio", "hash": "sha256:mno", "verified": true },
      "replication": { "path": "artifacts/cook/sector_0_0.replication", "hash": "sha256:pqr", "verified": true }
    }
  ],
  "duration_ms": 2100
}
```

### World generate output schema

**Formal schema:** `docs/specs/schemas/world_generate_result.schema.json`

```json
{
  "ok": true,
  "dry_run": true,
  "template": "starter_open_world",
  "template_version": 1,
  "name": "test_world",
  "output": "examples/generated_world_preview",
  "planned_files": [
    {
      "path": "examples/generated_world_preview/assets/worlds/test_world.ron",
      "kind": "world",
      "action": "create",
      "exists": false,
      "bytes": 4754,
      "hash": "sha256:68d3c4364800f876dc0fe033ec780e0b2a01f91d55d12dd2d4a0e0148368f144",
      "schema": "docs/specs/schemas/world.schema.json"
    }
  ],
  "planned_assets": [
    {
      "id": "test_world",
      "kind": "world",
      "path": "examples/generated_world_preview/assets/worlds/test_world.ron"
    }
  ],
  "diagnostics": [],
  "duration_ms": 4
}
```

### Profile summary output schema

**Formal schema:** `docs/specs/schemas/profile_summary_result.schema.json`

```json
{
  "ok": true,
  "artifact": "artifacts/profiles/open_world_traversal.trace",
  "capture_secs": 60,
  "duration_ms": 90,
  "frame": {
    "cpu": { "p50_ms": 9.1, "p95_ms": 13.2, "p99_ms": 15.4, "max_ms": 18.7 },
    "gpu": { "p50_ms": 7.8, "p95_ms": 11.5, "p99_ms": 13.9, "max_ms": 16.1 },
    "fps_average": 92
  },
  "sector_streaming": {
    "load_latency": { "p50_ms": 120, "p95_ms": 310, "p99_ms": 380, "max_ms": 420 },
    "crossing_hitch_ms": 5.2,
    "max_active_sectors": 49
  },
  "io": {
    "read_mb": 940,
    "requests": 420,
    "request_latency": { "p50_ms": 1.2, "p95_ms": 7.5, "p99_ms": 12.0, "max_ms": 20.0 }
  },
  "memory": { "peak_mb": 6200, "end_mb": 5800 },
  "replication": { "sent_kbps": 80, "received_kbps": 40, "relevancy_culled_entities": 12000 },
  "hitches": [],
  "budget_status": "pass"
}
```

### Eval output schema

**Formal schema:** `docs/specs/schemas/eval_report.schema.json`

```json
{
  "ok": true,
  "suite": "aaa_studio",
  "duration_ms": 120000,
  "pass_rate": 0.9,
  "tasks": [
    {
      "id": "EVAL-STUDIO-001",
      "passed": true,
      "repair_attempts": 1,
      "commands": [
        { "command": "aa index", "exit_code": 0, "duration_ms": 40 },
        { "command": "aa validate", "exit_code": 0, "duration_ms": 800 },
        { "command": "aa check", "exit_code": 0, "duration_ms": 2000 },
        { "command": "aa playtest", "exit_code": 0, "duration_ms": 45000 }
      ],
      "artifacts": { "log": "artifacts/evals/add_fire_ability.log", "diff": "artifacts/evals/final.patch" }
    }
  ]
}
```

### Eval list output schema

**Formal schema:** `docs/specs/schemas/eval_list_result.schema.json`

```json
{
  "ok": true,
  "suites": [
    {
      "id": "demo_game_add_fire_ability",
      "display_name": "Demo Game Add Fire Ability",
      "description": "Eval fixture for the first Cursor-like studio slice.",
      "tier": "studio_alpha",
      "path": "docs/specs/fixtures/demo_game/add_fire_ability.eval.json",
      "task_count": 1,
      "categories": ["ability"],
      "required_commands": ["aa index", "aa validate", "aa check", "aa ability graph", "aa playtest"],
      "min_pass_rate": 1.0,
      "max_repair_attempts": 3
    }
  ],
  "diagnostics": [],
  "duration_ms": 4
}
```

### Config get output schema

**Formal schema:** `docs/specs/schemas/config_get_result.schema.json`

```json
{
  "ok": true,
  "project": "examples/demo_game",
  "key": "net.default_port",
  "value": 7777,
  "value_type": "int",
  "source": "config/engine.toml",
  "diagnostics": [],
  "duration_ms": 3
}
```

---

## Playtest Scenario Schema

**File:** `assets/playtests/<name>.ron`
**Formal schema:** `docs/specs/schemas/playtest_scenario.schema.json`

```ron
PlaytestScenario(
    schema_version: 1,
    id: "fireball_hit",
    duration_secs: 30.0,
    setup: LoadScene("scenes/arena_test"),
    input_script: [
        (at_secs: 1.0, action: SpawnPlayer("pawns/hero_shooter")),
        (at_secs: 2.0, action: ActivateAbility("abilities/fireball")),
    ],
    assertions: [
        (name: "target_health_delta", check: AttributeDelta("Health", -25.0)),
        (name: "player_alive", check: EntityExists("player")),
    ],
)
```

## Agent Eval Schema

**File:** `assets/evals/<name>.ron`
**Formal schema:** `docs/specs/schemas/agent_eval.schema.json`

Eval assets define versioned prompt-to-feature tasks, required commands, acceptance checks, repair limits, and artifact expectations for `aa eval run`.

---

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-CLI-01 | validate good project | exit 0 | integration |
| T-CLI-02 | validate broken ref | exit 1, REF_MISSING | integration |
| T-CLI-03 | validate cycle prefab | exit 1, CYCLE_PREFAB | integration |
| T-CLI-04 | sarif format | valid SARIF 2.1 output with project-relative diagnostics | unit |
| T-CLI-05 | playtest smoke | exit 0 ≤ 45s | CI |
| T-CLI-06 | perf 1000 assets | ≤ 10s | bench |
| T-CLI-07 | scene patch dry-run | no file change | integration |
| T-CLI-08 | scene patch undo | restored state | integration |
| T-CLI-09 | world inspect fixture | sector/layer/cook JSON matches fixture | integration |
| T-CLI-10 | world generate dry-run | no file change, planned files returned | integration |
| T-CLI-11 | world cook deterministic | unchanged input yields same artifact hashes | integration |
| T-CLI-12 | eval report schema | machine-readable report validates | unit |
| T-CLI-13 | profile summarize fixture | hitch/sector summaries match fixture | unit |
| T-CLI-14 | command allowlist | world/eval commands cannot write outside project | integration |
| T-CLI-15 | playtest schema | malformed playtest scenario rejected | unit |
| T-CLI-16 | eval asset schema | malformed eval suite rejected | unit |
| T-CLI-17 | scene patch schema | malformed patch rejected before mutation | unit |
| T-CLI-18 | scene patch dry-run report | affected files/entities + undo token returned, no write, output validates against `schemas/scene_patch_result.schema.json` | integration |
| T-CLI-19 | asset manifest schema | malformed asset manifest rejected | unit |
| T-CLI-20 | world descriptor schema | malformed world descriptor rejected | unit |
| T-CLI-21 | world inspect missing sector | missing sector ref returns validation error | integration |
| T-CLI-22 | pawn data schema | malformed pawn asset rejected | unit |
| T-CLI-23 | input context schema | malformed input context rejected | unit |
| T-CLI-24 | action set schema | malformed action set rejected | unit |
| T-CLI-25 | attribute set schema | malformed attribute set rejected | unit |
| T-CLI-26 | unknown effect attribute | effect modifier attr not declared | validation error with `ATTR_UNKNOWN` |
| T-CLI-27 | AI profile schema | malformed AI profile rejected | unit |
| T-CLI-28 | spawn table schema | malformed spawn table rejected | unit |
| T-CLI-29 | spawn table refs | missing pawn/profile/prefab returns validation error | integration |
| T-CLI-30 | index result schema | query output validates schema | unit |
| T-CLI-31 | index relevant hits | `enemy camp sector` returns spec/schema/playtest/eval hits | integration |
| T-CLI-32 | validate result schema | JSON validate output validates schema | unit |
| T-CLI-33 | check result schema | JSON check output validates schema | unit |
| T-CLI-34 | playtest result schema | JSON playtest output validates schema | unit |
| T-CLI-35 | eval report schema | JSON eval output validates schema | unit |
| T-CLI-36 | world inspect result schema | JSON world inspect output validates schema | unit |
| T-CLI-37 | world cook result schema | JSON world cook output validates schema | unit |
| T-CLI-38 | profile summary result schema | JSON profile summarize output validates schema | unit |
| T-CLI-39 | open-world studio fixtures | `python3 docs/specs/tools/validate_contract_fixtures.py` exits 0; project-local CLI test later validates the same fixtures through `aa validate` | unit |
| T-CLI-40 | bootstrap validation bridge | `python3 docs/specs/tools/aa_bootstrap.py validate --format json` exits 0 and validates against `schemas/validation_result.schema.json`, including an initialized empty project fixture; `--format sarif` emits SARIF 2.1-shaped output validated by `schemas/sarif_2_1_min.schema.json` until `crates/aa_cli` owns the command | unit |
| T-CLI-41 | bootstrap index bridge | `python3 docs/specs/tools/aa_bootstrap.py index --query "enemy camp sector" --json` exits 0, validates against `schemas/index_result.schema.json`, and returns spec/schema/playtest/eval/asset hits until `crates/aa_cli` owns the command | unit |
| T-CLI-42 | bootstrap CLI tests | `python3 docs/specs/tools/test_bootstrap_cli.py` exits 0, covering manifest config/assets roots, initialized empty project validation, uninitialized directory diagnostics, project-relative diagnostics, unknown effect attributes, broken soft refs, project-local shim delegation, index result schema, scoped open-world asset hits, demo-game asset hits, and ability graph schema/diagnostics | unit |
| T-CLI-43 | bootstrap check bridge | `python3 docs/specs/tools/test_bootstrap_cli.py` covers `aa_bootstrap.py check` on passing and failing tiny Cargo projects, with output validating against `schemas/check_result.schema.json` | unit |
| T-CLI-44 | bootstrap eval bridge | `python3 docs/specs/tools/aa_bootstrap.py eval run open_world_studio_enemy_camp --json` exits 0 while validating against `schemas/eval_report.schema.json`; index, validate, check, scene inspect, scene patch dry-run, world inspect, world cook, playtest, profile, expected-file, forbidden-path, file-presence, and profile-budget checks all pass in the contract loop; a forbidden `target/` directory fails the eval | unit |
| T-CLI-45 | bootstrap world/playtest/profile fixture bridges | `aa_bootstrap.py world inspect`, `world cook`, `scene list`, `scene inspect`, `scene patch --dry-run`, `playtest --scenario smoke`, `playtest --scenario fireball_hit`, `playtest --scenario open_world_enemy_camp`, and `profile summarize` fixture outputs validate against their result schemas; these do not count as runtime gate proof | unit |
| T-CLI-46 | project-local bootstrap shim | `./aa validate --format json` exits 0 and matches the bootstrap validation result shape until the Rust CLI owns bare `aa` | unit |
| T-CLI-47 | bootstrap ability graph bridge | `aa_bootstrap.py ability graph basic_melee --project examples/open_world_studio --json` exits 0 and validates against `schemas/ability_graph_result.schema.json`; unregistered tags produce `TAG_UNREGISTERED` diagnostics | unit |
| T-CLI-48 | bootstrap demo-game studio eval | `aa_bootstrap.py eval run demo_game_add_fire_ability --json` exits 0 while validating against `schemas/eval_report.schema.json`; index, validate, check, ability graph, fireball playtest, expected-file, forbidden-path, file-presence, and allowlist checks all pass in the contract loop | unit |
| T-CLI-49 | bootstrap SARIF validation diagnostics | `aa_bootstrap.py validate <broken project> --format sarif` exits 1 and emits SARIF results with rule ids, severity levels, messages, and project-relative artifact URIs | unit |
| T-CLI-50 | bootstrap scene patch dry-run | `aa_bootstrap.py scene patch --scene examples/open_world_studio/assets/sectors/sector_0_0.ron --patch docs/specs/fixtures/open_world_studio/add_campfire.scene_patch.json --dry-run --json` exits 0, validates against `schemas/scene_patch_result.schema.json`, reports affected files/entities and an undo token, and leaves the sector file unchanged | unit |
| T-CLI-51 | bootstrap scene list/inspect | `aa_bootstrap.py scene list --scene examples/open_world_studio/assets/sectors/sector_0_0.ron --json` and `aa_bootstrap.py scene inspect sector_0_0/entity_0 --scene examples/open_world_studio/assets/sectors/sector_0_0.ron --json` exit 0 and validate against `schemas/scene_list_result.schema.json` / `schemas/scene_inspect_result.schema.json`; missing entity ids return `ENTITY_NOT_FOUND` | unit |
| T-CLI-52 | bootstrap unknown effect attribute | `aa_bootstrap.py validate <project-with-attributes-and-bad-effect> --format json|sarif` exits 1 and reports `ATTR_UNKNOWN` with the effect asset path | unit |
| T-CLI-53 | bootstrap world generate dry-run | `aa_bootstrap.py world generate --template starter_open_world --output examples/generated_world_preview --name test_world --dry-run --json` exits 0, validates against `schemas/world_generate_result.schema.json`, plans one world, nine sectors, and one traversal playtest, and leaves the output directory absent | unit |
| T-CLI-54 | bootstrap config get | `aa_bootstrap.py config get net.default_port --project examples/demo_game --json` exits 0, validates against `schemas/config_get_result.schema.json`, and reports the value source as `config/engine.toml`; unknown keys return `CONFIG_KEY_NOT_FOUND` | unit |
| T-CLI-55 | bootstrap eval list | `aa_bootstrap.py eval list --json` exits 0, validates against `schemas/eval_list_result.schema.json`, and lists the demo-game and open-world studio eval suites with task categories and required commands | unit |
| T-CLI-56 | bootstrap smoke playtest | `aa_bootstrap.py playtest --scenario smoke --json` exits 0, validates against `schemas/playtest_result.schema.json`, and returns log/trace artifact paths; this is contract proof only, not runtime playtest CI | unit |

---

## Acceptance

**P0 certified when:** T-CLI-01, 02, 05 green.

**P3 / AA studio certified when:** T-CLI-03–08 green + agent eval `add_fire_ability` PASS.

**AAA studio certified when:** T-CLI-09–14 and T-CLI-36–39 green + Gate AS pass in `04_ACCEPTANCE_GATES.md`.
