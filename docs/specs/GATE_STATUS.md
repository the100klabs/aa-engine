# Gate Status

> **Current evidence snapshot.** Gate status must be updated from command output and inspected artifacts, not intent.

## Snapshot

| Field | Value |
|-------|-------|
| Date | 2026-06-14 |
| Workspace | `aa_engine` (merged runtime + specs) |
| Repository | https://github.com/the100klabs/aa-engine |
| Git status | `f4fca97` — P0-03/04, P1-10 death_respawn, P1 gate PASS |
| AA CLI | `./aa` prefers Rust `aa_cli`; bootstrap fallback for remaining commands |
| `aa.project.toml` | Present at repo root and per example project |
| `config/*.toml` | Present at repo root and example projects |
| `crates/aa_*` | core, assets, scene, tags, input, ability, experience, gameplay, physics, animation, net (scaffold), **world_stream**, cli |
| `examples/demo_game` | Playable Phase 1 combat + `smoke`, `fireball_hit`, `locomotion_smoke`, `death_respawn` playtests |
| `examples/open_world_studio` | 256-sector (16×16 @ 256m) streamed world, 8 data layers, headless OWS playtests |
| Rust validate | `validate examples/open_world_studio --format json` passes (`diagnostics` schema) |
| Rust validate SARIF | `validate examples/demo_game --format sarif` emits SARIF 2.1 |
| Rust playtest | `smoke`, `fireball_hit`, `locomotion_smoke`, `death_respawn` pass on demo_game |
| OWS playtest | `open_world_enemy_camp` + `open_world_sector_traverse` pass (`sector_0_0=Active`, real spawn pipeline) |
| P1 unit tests | `cargo test -p aa_ability --test p1_gates` (3/3) |
| P1 integration | `cargo test -p aa_experience --test p1_gates` (P1-06), `cargo test -p aa_gameplay --test p1_gates` (P1-07) |
| World inspect | `aa world inspect` → **256 sectors**, **8 layers** (golden fixtures refreshed) |
| World cook | `aa world cook --verify --json` deterministic `artifacts/cook/*` |
| Index / eval / scene | Rust `aa index`, `aa eval run` (29/29 acceptance), `aa scene inspect/patch --dry-run` |
| Bootstrap CLI tests | `python3 docs/specs/tools/test_bootstrap_cli.py` **33/33** |

## Gate P0 - Foundation

| ID | Status | Evidence |
|----|--------|----------|
| P0-01 | PASS | `cargo clippy --workspace -- -D warnings` |
| P0-02 | PASS | `cargo test -p aa_core --test config_merge_order` (REQ-GLOBAL-050 all layers) |
| P0-03 | PASS | `cargo test -p aa_scene --test p0_gates spawn_player_prefab` (prefab root + child ≥ 3 components) |
| P0-04 | PASS | `cargo test -p aa_core --test schedule_ambiguity` (0 ambiguities, headless AA plugin stack) |
| P0-05 | PASS | Rust `aa validate` schema subset (world/sector/spawn_table/ability) + SARIF + prefab refs |

**GATE: FAIL** (P0-06/07 not yet evidenced)

## Gate P1 - Combat Vertical Slice

| ID | Status | Evidence |
|----|--------|----------|
| P1-01 | PASS | `locomotion_smoke` playtest passes (movement intent injected) |
| P1-02 | PASS | Aim-cone fireball hitscan + `fireball_hit` playtest |
| P1-03 | PASS | `effect_modifies_attribute` unit test |
| P1-04 | PASS | `asc_on_player_state` unit test |
| P1-05 | PASS | `stun_blocks_fire` unit test |
| P1-06 | PASS | `cargo test -p aa_experience --test p1_gates` — `ExperienceReady` + ability grants resolve |
| P1-07 | PASS | `cargo test -p aa_gameplay --test p1_gates` — attribute set + input context + `PendingInit` cleared |
| P1-08 | PASS | `cargo test -p aa_ability --test p1_gates data_driven_abilities_ron_only_audit` (4 RON abilities) |
| P1-09 | PASS | `bench_100_asc` criterion bench in `aa_ability` |
| P1-10 | PASS | `death_respawn` playtest — move, fire, die to dummy melee, respawn with Health=100 |

**GATE: PASS**

## Gate OWA - Open World Alpha

| ID | Status | Evidence |
|----|--------|----------|
| OWA-01 | PASS | 16×16 sector grid @ 256m = 16 km² (256 sectors) |
| OWA-02 | PASS | Runtime `aa world inspect --live` snapshots `SectorRegistry` via headless app |
| OWA-03 | PASS | 8 data layers in world descriptor |
| OWA-04 | PASS | `open_world_enemy_camp` + `open_world_sector_traverse` playtests in CI |
| OWA-05 | PASS | `aa world cook --verify` deterministic artifacts |
| OWA-06 | PASS | `aa profile summarize` on playtest trace JSON |
| OWA-07 | PASS | Spawn table pipeline activates `camp_guard_patrol` without fallback |
| OWA-08 | PASS | `aa eval run open_world_studio_elemental_ability` + `basic_ranged_attack.ron` |

**GATE: PASS**

## Gate AS - Agent Studio

| ID | Status | Evidence |
|----|--------|----------|
| AS-01 | PASS | `aa eval run open_world_studio_enemy_camp` — commands + 29 acceptance checks |
| AS-03 | PASS | `aa index --query`, `aa scene patch --dry-run` in Rust |

**GATE: PARTIAL**

## Gate P3 - Studio / Agent (legacy IDs)

| ID | Status | Evidence |
|----|--------|----------|
| P3-01 | PASS | Rust `aa validate` JSON |
| P3-02 | PASS | Rust `aa validate --format sarif` |
| P3-03 | PASS | Runtime playtest CI (`smoke`, `fireball_hit`, `locomotion_smoke`, `death_respawn`, OWS) |
| P3-04 | PASS | Rust `aa index --query` |
| P3-05 | PASS | Rust `aa eval list/run` with acceptance parity |

**GATE: PARTIAL**

## Commands (success criteria)

```bash
cargo run -p aa_cli -- world inspect --project examples/open_world_studio --world open_world_studio --json
cargo run -p aa_cli -- world inspect --project examples/open_world_studio --world open_world_studio --live --json
cargo run -p aa_cli -- world cook --project examples/open_world_studio --world open_world_studio --verify --json
cargo run -p aa_cli -- playtest --project examples/open_world_studio --scenario open_world_enemy_camp --duration 25 --json
cargo run -p aa_cli -- playtest --project examples/open_world_studio --scenario open_world_sector_traverse --duration 20 --json
cargo run -p aa_cli -- validate examples/open_world_studio --format json
cargo run -p aa_cli -- index --query "enemy camp sector" --json
cargo run -p aa_cli -- eval run open_world_studio_enemy_camp --json
cargo run -p aa_cli -- eval run open_world_studio_elemental_ability --json
cargo test -p aa_experience --test p1_gates
cargo test -p aa_gameplay --test p1_gates
cargo test -p aa_core --test config_merge_order
cargo test -p aa_core --test schedule_ambiguity
cargo test -p aa_scene --test p0_gates spawn_player_prefab
python3 docs/specs/tools/test_bootstrap_cli.py
```
