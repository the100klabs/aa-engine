# 06 - AAA Open World Studio Track

> **Normative for post-AA ambition.** This document defines the measurable path from AA Engine to a AAA-capable open-world engine and Cursor-like game studio. It does not weaken the AA gates in `04_ACCEPTANCE_GATES.md`; the AA product must still ship first.

## Product Thesis

The engine becomes "AAA open world" only when two things are true at the same time:

1. The runtime can stream, simulate, render, and network large worlds without hitches.
2. The studio can let humans and agents create, inspect, validate, playtest, and repair those worlds through structured tools.

"Cursor for games" is not a chat box pasted onto an editor. It is a closed production loop:

```text
intent -> indexed context -> generated patch -> validation -> playtest -> diagnostics -> repair -> accepted diff
```

The loop MUST work for code, gameplay assets, scenes, streamed sectors, abilities, effects, input, AI, quests, and performance budgets.

## Scope Boundary

| In scope | Out of scope |
|----------|--------------|
| Large streamed worlds | UE5 source compatibility |
| Agent-safe editing and repair | Blueprint clone as primary workflow |
| Deterministic validation/playtests | Film-tier virtual production parity |
| High-density gameplay simulation | MetaHuman/facial pipeline parity |
| Data-driven authoring | Marketplace/FAB clone |
| Scalable runtime budgets | Console certification before AA |

## Target Tiers

| Tier | Code | Definition | Blocks |
|------|------|------------|--------|
| Studio Alpha | SA | Agents can safely add gameplay features to a small streamed world | Public studio preview |
| Open World Alpha | OWA | 16 km2 world with cooked sectors, nav, HLOD, and playtests | Open-world sample |
| Open World Beta | OWB | 64 km2 world with multiplayer relevancy and authoring tools | AAA-capable claim |
| AAA Studio | AS | Prompt-to-playable feature loop passes broad eval suite | "Cursor for games" claim |

## Non-Negotiable Strategy

1. Build the `aa_cli` contract before the GUI. Agents need JSON/SARIF diagnostics, scene patching, playtests, and undo tokens before they can work safely.
2. Keep gameplay data text-first. RON/TOML remains the source of truth for assets that agents edit.
3. Partition every large world. A giant scene file is a blocker for open-world certification.
4. Make every agent action verifiable. A generated feature is not accepted until validation and a relevant playtest pass.
5. Expose runtime state through structured inspection APIs. Agents must not infer game state from logs alone.
6. Treat performance as a content error. Sector, HLOD, nav, and replication budgets must fail validation or playtests when violated.

## Runtime Requirements

| ID | Requirement |
|----|-------------|
| REQ-AAA-RUN-001 | Worlds over 4 km2 MUST use hierarchical partitioning: region -> sector -> subcell. |
| REQ-AAA-RUN-002 | Sector descriptors MUST be text assets with `schema_version` and soft refs. |
| REQ-AAA-RUN-003 | Runtime streaming MUST support multiple streaming sources. |
| REQ-AAA-RUN-004 | Sector activation MUST be budgeted by priority, distance, gameplay layer, and frame time. |
| REQ-AAA-RUN-005 | Sector load/unload MUST leave no orphan entity, physics, ability, nav, or net state. |
| REQ-AAA-RUN-006 | Sector cook MUST produce deterministic artifacts for render, physics, nav, spawn, audio, and replication metadata. |
| REQ-AAA-RUN-007 | Terrain MUST support tiled height/mesh data with collision and nav generation per tile. |
| REQ-AAA-RUN-008 | Static environment density MUST use instancing and HLOD, not one entity per visual prop at long range. |
| REQ-AAA-RUN-009 | AI simulation MUST support LOD policies: full, simplified, dormant, and despawned. |
| REQ-AAA-RUN-010 | Network relevancy MUST be sector-aware and MUST NOT broadcast distant open-world state. |
| REQ-AAA-RUN-011 | Save/load MUST persist player and world deltas separately from authored sector baselines. |
| REQ-AAA-RUN-012 | Runtime diagnostics MUST expose per-sector CPU, GPU, memory, IO, entity, nav, and replication costs. |
| REQ-AAA-RUN-013 | Open-world projects MUST define worlds with assets validated by `docs/specs/schemas/world.schema.json`. |
| REQ-AAA-RUN-014 | Enemy population MUST use spawn table assets and AI profile assets, not hardcoded Rust spawn lists. |

## Studio Requirements

| ID | Requirement |
|----|-------------|
| REQ-AAA-STUDIO-001 | `aa index --query` MUST return relevant specs, assets, systems, schemas, and playtests for an agent task. |
| REQ-AAA-STUDIO-002 | `aa scene patch` MUST support streamed-sector patches with dry-run validation and undo tokens. |
| REQ-AAA-STUDIO-003 | `aa world inspect` MUST report sector bounds, layers, refs, cooked artifacts, budgets, and live activation state as `docs/specs/schemas/world_inspect_result.schema.json`. |
| REQ-AAA-STUDIO-004 | `aa world generate` MUST create valid starter regions from templates, never unvalidated giant scenes. |
| REQ-AAA-STUDIO-005 | `aa playtest` MUST support camera, input, bot, and assertion scripts for open-world traversal. |
| REQ-AAA-STUDIO-006 | Agent-created gameplay features MUST include or update at least one playtest scenario. |
| REQ-AAA-STUDIO-007 | The studio MUST show proposed diffs for code, RON/TOML, scene patches, and generated world assets before acceptance. |
| REQ-AAA-STUDIO-008 | The repair loop MUST consume structured diagnostics from check, validate, playtest, world inspect, world cook, and profile runs. |
| REQ-AAA-STUDIO-009 | The editor and CLI MUST share the same patch/undo backend. |
| REQ-AAA-STUDIO-010 | Agents MUST operate inside allowlisted project paths and MUST NOT write generated patches directly to `target/` or `.git/`. |
| REQ-AAA-STUDIO-011 | Prompt-to-feature evaluations MUST be versioned assets in `assets/evals/` or an equivalent test directory. |
| REQ-AAA-STUDIO-012 | Studio telemetry MUST measure task success, repair attempts, validation failures, and human reverts. |

## Authoring Primitives

| Need | Primitive |
|------|-----------|
| New game | Project template with `aa.project.toml`, default assets, smoke playtest |
| New world | WorldDescriptor asset plus region template, sector grid, and data layers |
| New biome | Layered sector patch, material set, foliage rules, spawn tables |
| New enemy | PawnData, AI profile, abilities/effects, spawn table, playtest |
| New ability | Ability RON, effect RON, tags, Rust registrar, graph validation |
| New quest/activity | Data asset, trigger volumes, rewards, assertions |
| Perf fix | Profile capture, sector budget report, generated HLOD or density patch |

## Cursor-Like Workflow Contract

Every accepted agent task MUST pass through these states:

| State | Tool proof |
|-------|------------|
| Context gathered | `aa index --query <task>` returns ranked hits validated by `schemas/index_result.schema.json` |
| Patch proposed | Structured diff or scene patch with affected files/entities |
| Static validation | `aa validate --format json` exits 0 with output validated by `schemas/validation_result.schema.json`; SARIF is also available for CI/editor integrations |
| Compile check | `aa check --json` exits 0 with output validated by `schemas/check_result.schema.json` for Rust changes |
| Domain graph check | Ability/world/scene graph command exits 0 when relevant |
| Playtest | Relevant `aa playtest --scenario <id> --json` exits 0 with output validated by `schemas/playtest_result.schema.json` |
| Artifact captured | Logs, traces, screenshots, or profile paths recorded |
| Open-world inspection | `aa world inspect --json` output validates against `schemas/world_inspect_result.schema.json` when world content changes |
| Performance summary | `aa profile summarize <artifact_path> --json` output validates against `schemas/profile_summary_result.schema.json` when performance is relevant |

If a task changes gameplay behavior and has no playtest, the studio MUST treat the task as incomplete.

## Open-World Performance Targets

These targets extend `02_PERFORMANCE_BUDGETS.md` after AA certification.

| Metric | Studio Alpha | Open World Alpha | Open World Beta |
|--------|--------------|------------------|-----------------|
| Authored world size | 4 km2 | 16 km2 | 64 km2 |
| Active sectors | 5x5 | 7x7 | policy-driven multi-source |
| Sector load p95 | <= 500 ms | <= 400 ms | <= 300 ms |
| Sector crossing hitch | <= 8 ms | <= 6 ms | <= 4 ms |
| Placed authored objects | 25k | 250k | 1M |
| Visible instanced props | 10k | 100k | 500k |
| Full AI agents | 100 | 500 | 1,000 |
| Low-LOD agents | 1,000 | 5,000 | 10,000 |
| Multiplayer players | 8 | 16 | 32 |
| Validate time | <= 10s @ 1000 assets | <= 20s @ 10k assets | <= 60s @ 100k assets |

## Evaluation Suite

Golden fixture examples for the first open-world studio task live under `docs/specs/fixtures/open_world_studio/`.
They define the expected eval input, playtest input, inspect output, cook output, and profile output for the `add_enemy_camp` loop.

The studio is not "Cursor-like" until these tasks pass without hand edits in a controlled eval:

Eval suites MUST be versioned assets validated by `docs/specs/schemas/agent_eval.schema.json`. Eval reports MUST validate against `docs/specs/schemas/eval_report.schema.json`.

| ID | Prompt class | Required result |
|----|--------------|-----------------|
| EVAL-STUDIO-001 | Add a damage ability | RON assets, tags, registrar, graph, playtest pass |
| EVAL-STUDIO-002 | Add an enemy camp | Sector patch, spawn table, nav validation, traversal playtest |
| EVAL-STUDIO-003 | Add a biome layer | Data layer, foliage/prop rules, streaming budget pass |
| EVAL-STUDIO-004 | Fix a broken soft ref | Validation diagnostic consumed, asset corrected, validate pass |
| EVAL-STUDIO-005 | Fix compile error | `aa check --json` diagnostic consumed, code corrected, check pass |
| EVAL-STUDIO-006 | Reduce sector hitch | Profile consumed, content patch generated, crossing playtest pass |
| EVAL-STUDIO-007 | Add multiplayer pickup | Server authority, replication manifest, net playtest pass |
| EVAL-STUDIO-008 | Create a new project | Template generated, smoke playtest pass |

Minimum pass rates:

| Tier | Pass rate | Max repair attempts |
|------|-----------|---------------------|
| Studio Alpha | 60% | 5 |
| Open World Alpha | 70% | 4 |
| Open World Beta | 80% | 3 |
| AAA Studio | 90% | 2 |

## Reference Open-World Game

The AAA/Open World track MUST include:

**`examples/open_world_studio/`**

Required proof:

- 16 km2 world for OWA, 64 km2 for OWB.
- At least 64 authored sectors.
- At least 8 named data layers.
- Traversal, combat, spawn, sector crossing, save/load, and multiplayer smoke playtests.
- Agent-completable task: "add a new enemy camp to sector X/Y."
- Agent-completable task: "add a new elemental ability and make one enemy use it."
- Performance profile artifacts for representative traversal.

No AAA/Open World claim is allowed until this example passes its gate in `04_ACCEPTANCE_GATES.md`.

## Implementation Order

1. `aa_cli` P0: check, validate, index, playtest, scene patch.
2. `aa_assets` and `aa_scene`: manifest, schemas, soft refs, patchable scenes.
3. `aa_gameplay` and `aa_ability`: data-driven vertical slice with playtests.
4. `aa_world_stream`: sectors, layers, streaming policy, inspection.
5. `aa_editor`: shared patch backend and live viewport inspection.
6. World cook pipeline: deterministic sector cook artifacts exposed through `aa world cook`.
7. Agent eval harness: versioned prompt-to-feature evals exposed through `aa eval run`.
8. `examples/open_world_studio`: reference proof game.

## Completion Rule

The AAA/Open World objective is achieved only when:

1. AA certification gates pass.
2. Open-world gates pass.
3. Studio eval gates pass.
4. The reference open-world game passes all required playtests.
5. Performance artifacts prove the published budgets.
6. Agents can complete the required feature tasks through the CLI/editor loop without hand edits.
