# Gate Status

> **Current evidence snapshot.** Gate status must be updated from command output and inspected artifacts, not intent.

## Snapshot

| Field | Value |
|-------|-------|
| Date | 2026-06-14 |
| Workspace | `aa_engine` (merged runtime + specs) |
| Repository | https://github.com/the100klabs/aa-engine |
| Git status | AAA continuation: 16 km² world, spawn tables, Rust CLI studio loop, OWS playtests |
| AA CLI | `./aa` prefers Rust `aa_cli`; bootstrap fallback for remaining commands |
| `aa.project.toml` | Present at repo root and per example project |
| `config/*.toml` | Present at repo root and example projects |
| `crates/aa_*` | core, assets, scene, tags, input, ability, experience, gameplay, physics, animation, net (scaffold), **world_stream**, cli |
| `examples/demo_game` | Playable Phase 1 combat + `smoke`, `fireball_hit`, `locomotion_smoke` playtests |
| `examples/open_world_studio` | 256-sector (16×16 @ 256m) streamed world, 8 data layers, headless OWS playtests |
| Rust validate | `cargo run -p aa_cli -- validate examples/demo_game --format json` passes |
| Rust validate SARIF | `cargo run -p aa_cli -- validate examples/demo_game --format sarif` emits SARIF 2.1 |
| Rust playtest | `smoke`, `fireball_hit`, `locomotion_smoke` pass on demo_game |
| OWS playtest | `open_world_enemy_camp` passes with streaming + spawn assertions |
| P1 unit tests | `cargo test -p aa_ability --test p1_gates` (3/3) |
| World inspect | `aa world inspect --project examples/open_world_studio --world open_world_studio` → 256 sectors, 8 layers |
| World cook | `aa world cook --verify --json` writes deterministic `artifacts/cook/*` |
| Index / eval / scene | `aa index`, `aa eval list`, `aa scene list/inspect/patch --dry-run` in Rust CLI |
| Bootstrap CLI tests | `python3 docs/specs/tools/test_bootstrap_cli.py` (fixture drift possible on 256-sector world) |

## Gate P0 - Foundation

| ID | Status | Evidence |
|----|--------|----------|
| P0-01 | PASS | `cargo clippy --workspace -- -D warnings` |
| P0-02 | PARTIAL | Config layers exist; full merge test suite pending |
| P0-05 | PARTIAL | Rust `aa validate` JSON + SARIF |

**GATE: FAIL**

## Gate P1 - Combat Vertical Slice

| ID | Status | Evidence |
|----|--------|----------|
| P1-01 | PASS | `locomotion_smoke` playtest passes (movement intent injected) |
| P1-02 | PASS | Aim-cone fireball hitscan + `fireball_hit` playtest |
| P1-03 | PASS | `effect_modifies_attribute` unit test |
| P1-04 | PASS | `asc_on_player_state` unit test |
| P1-05 | PASS | `stun_blocks_fire` unit test |
| P1-09 | PASS | `bench_100_asc` criterion bench in `aa_ability` |

**GATE: FAIL** (integration gates P1-06–10 still open)

## Gate OWA - Open World Alpha

| ID | Status | Evidence |
|----|--------|----------|
| OWA-01 | PASS | 16×16 sector grid @ 256m = 16 km² (256 sectors) |
| OWA-02 | PARTIAL | Runtime `aa world inspect` + `--live` stub |
| OWA-03 | PASS | 8 data layers in world descriptor |
| OWA-04 | PARTIAL | Sector load/unload + `open_world_enemy_camp` playtest |
| OWA-05 | PASS | `aa world cook --verify` deterministic artifacts |
| OWA-06 | PASS | `aa profile summarize` on playtest trace JSON |
| OWA-07 | PARTIAL | Spawn table pipeline + camp guard playtest assertion |
| OWA-08 | PARTIAL | `add_elemental_ability` eval fixture + `basic_ranged_attack.ron` |

**GATE: PARTIAL**

## Gate AS - Agent Studio

| ID | Status | Evidence |
|----|--------|----------|
| AS-01 | PARTIAL | `aa eval run open_world_studio_enemy_camp` executes Rust CLI commands |
| AS-03 | PARTIAL | `aa index --query`, `aa scene patch --dry-run` in Rust |

**GATE: PARTIAL**

## Gate P3 - Studio / Agent (legacy IDs)

| ID | Status | Evidence |
|----|--------|----------|
| P3-01 | PASS | Rust `aa validate` JSON |
| P3-02 | PASS | Rust `aa validate --format sarif` |
| P3-03 | PASS | Runtime playtest CI (`smoke`, `fireball_hit`, `locomotion_smoke`) |
| P3-04 | PARTIAL | Rust `aa index --query` |
| P3-05 | PARTIAL | Rust `aa eval list/run` |

**GATE: PARTIAL**

## Commands (success criteria)

```bash
cargo run -p aa_cli -- world inspect --project examples/open_world_studio --world open_world_studio --json
cargo run -p aa_cli -- world cook --project examples/open_world_studio --world open_world_studio --verify --json
cargo run -p aa_cli -- playtest --project examples/open_world_studio --scenario open_world_enemy_camp --duration 20
cargo run -p aa_cli -- validate examples/open_world_studio --format json
cargo run -p aa_cli -- index --query "enemy camp sector" --json
cargo run -p aa_cli -- eval run open_world_studio_enemy_camp --json
```
