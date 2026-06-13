# 07 - Engine Studio Execution Plan

> **Normative planning artifact.** This document turns the AA and AAA/Open World specs into an implementation backlog. It is not a substitute for subsystem specs; it orders the work so the engine can become easy to build with in the Cursor-like way described by `06_AAA_OPEN_WORLD_STUDIO.md`.

## Current Baseline

As of 2026-06-13, this checkout contains the Bevy workspace, AA specs, a minimal root `aa.project.toml`, and baseline `config/*.toml` files, but it does not yet contain:

- implemented `crates/aa_*` runtime crates
- implemented project-local `aa_cli`
- `examples/lyra_equivalent`
- a runtime-streaming `examples/open_world_studio` Bevy package
- passing gates in `docs/specs/GATE_STATUS.md`

The `/usr/bin/aa` command available on macOS is Apple Archive, not the AA Engine CLI. Do not treat it as evidence for `aa check`, `aa validate`, or `aa playtest`.

## Build Principle

The fastest route to "super fast games very easily" is not to start with rendering spectacle. It is to build the reliable agent loop first:

```text
project metadata -> index -> patch -> validate -> check -> playtest -> inspect/profile -> repair
```

Every engine feature should become easier to build because the loop can see it, edit it, validate it, run it, and explain failures.

## Workstream Order

| Order | Workstream | Why it comes here |
|-------|------------|-------------------|
| 1 | Project metadata | Agents need project structure before they can edit safely |
| 2 | `aa_cli` P0 | Agents need stable JSON/SARIF commands before GUI/editor work |
| 3 | `aa_assets` + schemas | Text-first gameplay data needs validation and soft refs |
| 4 | `aa_scene` patching | Cursor-like scene edits need dry-run and undo |
| 5 | `aa_gameplay` vertical slice | Possession, PlayerState, PawnData, and respawn prove the gameplay skeleton |
| 6 | `aa_ability` vertical slice | Abilities/effects/tags prove data-driven gameplay authoring |
| 7 | `aa_playtest` scenarios through CLI | Every feature needs executable proof |
| 8 | `aa_world_stream` P2 | Sector streaming becomes the open-world foundation |
| 9 | `aa_editor` shared patch backend | GUI and CLI must edit through the same safe path |
| 10 | Open-world cook/profile/eval loop | AAA/Open World gates need scale, artifacts, and agent evals |

## First 30 Days

| Day range | Deliverable | Required proof |
|-----------|-------------|----------------|
| 1-2 | Add root `aa.project.toml` schema and sample | `aa validate` design documented; no absolute asset paths |
| 3-7 | Scaffold `crates/aa_cli` with `check`, `validate`, `--json`, exit codes | CLI integration tests for success/failure JSON |
| 8-12 | Implement asset manifest and schema validation path | Broken soft ref test returns `REF_MISSING` |
| 13-17 | Add `examples/demo_game` minimum contract project | `./aa validate examples/demo_game --format json` exits 0 |
| 18-22 | Add scene list/inspect/patch dry-run | Patch dry-run returns planned edits and undo token |
| 23-27 | Add smoke playtest harness | `aa playtest --scenario smoke --json` returns pass/fail and log path |
| 28-30 | Record P0 gate attempt | `docs/specs/GATE_STATUS.md` updated with command output evidence |

Current bootstrap bridge: `python3 docs/specs/tools/aa_bootstrap.py validate --format json`
validates the root manifest, the initialized empty project fixture, baseline
config files, and known RON asset fixture types, including the open-world
pawn/prefab/ability/effect/tag assets. It emits `schemas/validation_result.schema.json`
and returns `REF_MISSING` for broken spawn-table soft refs. It also reports
`ATTR_UNKNOWN` when a GameplayEffect modifier targets an attribute not declared
by any AttributeSet asset in the project. The same bridge now supports
`--format sarif`, emitting SARIF 2.1-shaped diagnostics validated by
`schemas/sarif_2_1_min.schema.json` for editor/CI integrations. This should be
ported into `crates/aa_cli` once adding the crate is approved.

The project-local `./aa` shim delegates to the same bootstrap implementation so
agents can invoke `./aa validate`, `./aa check`, `./aa playtest`, and
`./aa eval run` today. Bare `aa` still resolves to Apple Archive on macOS.

The companion bootstrap index bridge
`python3 docs/specs/tools/aa_bootstrap.py index --query "enemy camp sector" --json`
emits `schemas/index_result.schema.json`, includes editable
`examples/open_world_studio/assets` RON hits, and should become the first slice
of `aa index --query`.

The bootstrap config and eval discovery bridges
`python3 docs/specs/tools/aa_bootstrap.py config get net.default_port --project examples/demo_game --json`
and `python3 docs/specs/tools/aa_bootstrap.py eval list --json`
emit `schemas/config_get_result.schema.json` and
`schemas/eval_list_result.schema.json`. They let agents inspect project settings
with source attribution and enumerate available prompt-to-feature suites before
generating patches.

Bootstrap behavior is covered by `python3 docs/specs/tools/test_bootstrap_cli.py`,
including custom `config_root` / `assets_root` values from `aa.project.toml`,
initialized empty project validation, uninitialized directory diagnostics, and
scoped asset hits for the open-world project. The same test file covers broken
spawn-table refs, the local shim, and the bootstrap check bridge on passing and
failing tiny Cargo projects,
producing `schemas/check_result.schema.json` output for both paths.

The bootstrap eval bridge
`python3 docs/specs/tools/aa_bootstrap.py eval run open_world_studio_enemy_camp --json`
produces `schemas/eval_report.schema.json`. It exits 0 for the contract loop:
bootstrap index, validate, check, scene inspect, scene patch dry-run, world inspect, world cook,
playtest, and profile steps are all green. The report also records acceptance checks for
expected files, forbidden paths, authored file presence, and profile budgets.
The bootstrap tests include a negative case proving a generated `target/`
directory under the task project fails the eval.
This proves the agent workflow shape, not runtime sector streaming.

The bootstrap smoke playtest bridge
`python3 docs/specs/tools/aa_bootstrap.py playtest --scenario smoke --json`
emits `schemas/playtest_result.schema.json` output with log/trace artifact paths.
It proves the mandatory agent completion command is callable through the local
shim; it does not replace the runtime smoke playtest or CI gate.

The bootstrap open-world fixture bridges return the golden inspect, cook,
world generate dry-run, scene list/inspect, scene patch dry-run, playtest, and profile summary outputs for
`open_world_studio`. They are contract tests for JSON shape, patch safety, and
agent workflow integration; world generation plans one world, nine sectors, and
one traversal smoke playtest from a versioned starter template without writing.
They do not prove sector streaming, mutation/undo,
cooking, playtest, or profiling runtime behavior.

The bootstrap ability graph bridge
`python3 docs/specs/tools/aa_bootstrap.py ability graph basic_melee --project examples/open_world_studio --json`
emits `schemas/ability_graph_result.schema.json`. It reports the ability asset,
registrar, Gameplay Tags, cost effect when present, and AI profiles that consume
the ability. Bootstrap tests cover both the valid `basic_melee` graph and
`TAG_UNREGISTERED` diagnostics for missing tag dictionary entries.

`examples/demo_game` now contains the first studio contract slice: config TOML,
PawnData, AttributeSet, InputContext, ActionSet, ExperienceDefinition, fireball
ability/effect/tag assets, a `fireball_hit` playtest asset, and a standalone Rust
entry point that compile-checks the referenced files. The bootstrap index,
validate, check, ability graph, playtest, and eval bridges can run the
`demo_game_add_fire_ability` contract loop. This is contract proof for the
Cursor-like workflow shape, not yet a Bevy runtime combat demo.

`examples/open_world_studio` now contains the first contract-data slice:
project/config TOML, world, sector, spawn table, AI profile, playtest RON
assets, and a standalone Rust entry point that compile-checks those assets. It
is deliberately isolated from the root workspace until a workspace crate is
approved.

## First Playable Slice

The first playable slice exists when:

1. `examples/demo_game` boots.
2. A player pawn spawns from a text asset.
3. Controller possession uses `Possesses` / `PossessedBy`.
4. Human ability state lives on `PlayerState`.
5. One ability applies one `GameplayEffect`.
6. A smoke playtest proves move, activate ability, and quit.
7. `aa check`, `aa validate`, and `aa playtest --scenario smoke` are all green.

Do not start the open-world reference game until this slice is true.

## First Open-World Slice

The first open-world slice exists when:

1. A 3x3 sector test world loads around one streaming source.
2. Walking across three sectors does not crash.
3. Layer toggles hide/show sector content.
4. Sector unload leaves no entity, physics, ability, nav, audio, or net orphan state.
5. `aa world inspect` reports sector count, layers, refs, cook status, and budgets.
6. A sector traversal playtest captures a profile artifact.

This is the bridge from AA streamed arena to open-world authoring.

## First Cursor-Like Studio Slice

The first Cursor-like studio slice exists when an agent can complete this eval without hand edits:

```text
Add a fire ability to demo_game.
```

Required proof:

- `aa index --query "fire ability"` returns relevant specs/assets/code.
- The patch adds ability RON, effect RON, tag entries, and registrar code.
- `aa validate --format sarif` exits 0.
- `aa check --json` exits 0.
- `aa ability graph <ability_id> --json` exits 0.
- `aa playtest --scenario fireball_hit --json` exits 0.
- The eval report records commands, artifacts, repair attempts, and final diff.

## First AAA/Open World Studio Slice

The first AAA/Open World studio slice exists when an agent can complete this eval without hand edits:

```text
Add an enemy camp to sector 0/0 in open_world_studio.
```

Required proof:

- `aa index --query "enemy camp sector"` returns relevant sector, spawn, AI, and playtest docs.
- `aa world inspect --world open_world_studio --json` reports the target sector.
- The patch adds or updates sector data, spawn tables, AI profile refs, and playtest assertions.
- `aa validate --format sarif` exits 0.
- `aa world cook --world open_world_studio --verify --json` exits 0.
- `aa playtest --scenario open_world_enemy_camp --json` exits 0.
- `aa profile summarize <artifact_path> --json` shows budgets remain inside the current gate.

Golden fixture inputs and outputs for this loop live in `docs/specs/fixtures/open_world_studio/`.
They should become CLI contract tests when the project-local `aa_cli` crate is implemented.

## Parallel Work Policy

Parallelize only after the CLI contracts are stable:

- Runtime crates can develop in parallel once `aa check` and `aa validate` exist.
- Editor UI can develop in parallel once scene/world patch APIs exist.
- Rendering upgrades can develop in parallel once world streaming diagnostics expose budget data.
- Agent evals can develop in parallel once playtests and structured diagnostics exist.

## Stop Conditions

Pause feature expansion and repair the platform when any of these happen:

- `aa validate` cannot explain a broken asset in machine-readable output.
- A gameplay change has no playtest path.
- A scene/world patch cannot be dry-run or undone.
- A sector unload leaves stale runtime state.
- An agent task succeeds only by editing outside the project allowlist.
- Performance regressions are visible only to humans and not captured in artifacts.

## Done Means

The original objective is not complete until:

1. AA gates pass.
2. OWA, OWB, and AS gates pass.
3. `examples/open_world_studio` proves large-world authoring, traversal, combat, save/load, multiplayer smoke, and profiling.
4. Agents can complete the required eval suite with the pass rates in `06_AAA_OPEN_WORLD_STUDIO.md`.
5. The engine has verified artifacts for check, validate, playtest, inspect, cook, profile, and eval.
