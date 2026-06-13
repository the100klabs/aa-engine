# Gate Status

> **Current evidence snapshot.** Gate status must be updated from command output and inspected artifacts, not intent.

## Snapshot

| Field | Value |
|-------|-------|
| Date | 2026-06-13 |
| Workspace | `aa_engine` (merged runtime + specs) |
| Repository | https://github.com/the100klabs/aa-engine |
| Git status | Initial publish |
| AA CLI | `./aa` bootstrap shim + `cargo run -p aa_cli` Rust CLI |
| `aa.project.toml` | Present at repo root and per example project |
| `config/*.toml` | Present at repo root and example projects |
| `crates/aa_*` | Implemented: core, assets, scene, tags, input, ability, experience, gameplay, physics, animation, net (scaffold), cli |
| `examples/demo_game` | **Playable** Phase 1 combat slice (move, aim, 4 abilities, damage, respawn) + headless playtest |
| `examples/open_world_studio` | Open-world **contract** package (world/sector/spawn/AI RON assets) |
| `examples/lyra_equivalent` | Not started |
| Rust validate | `cargo run -p aa_cli -- validate examples/demo_game` passes |
| Rust playtest | `cargo run -p aa_cli -- playtest --project examples/demo_game --scenario smoke` passes (runtime harness) |
| Bootstrap CLI tests | `python3 docs/specs/tools/test_bootstrap_cli.py` (run in CI) |

## Gate P0 - Foundation

| ID | Status | Evidence |
|----|--------|----------|
| P0-01 | PASS | `cargo check --workspace` and `cargo clippy --workspace -- -D warnings` |
| P0-02 | PARTIAL | Config layers exist; full merge test suite pending |
| P0-03 | PARTIAL | Prefab spawn via `aa_scene`; integration test pending |
| P0-04 | PARTIAL | `AaSchedule` registered; ambiguity test pending |
| P0-05 | PARTIAL | Rust `aa validate` passes on demo_game; JSON schema validation via `./aa validate` bootstrap |
| P0-06 | UNKNOWN | Manual platform boot not recorded in CI |
| P0-07 | FAIL | REQ-to-test traceability incomplete |

**GATE: FAIL**

## Gate P1 - Combat Vertical Slice

| ID | Status | Evidence |
|----|--------|----------|
| P1-01 | FAIL | `locomotion_smoke` playtest not implemented |
| P1-02 | PARTIAL | Runtime smoke playtest passes; dedicated `fireball_hit` (−25 Health) not proven without fallback |
| P1-03 | PARTIAL | Damage via `GameplayEffect`; unit `effect_modifies_attribute` missing |
| P1-04 | PARTIAL | ASC on `PlayerState` in runtime; unit test missing |
| P1-05 | FAIL | `stun_blocks_fire` unit test missing |
| P1-06 | PARTIAL | `ExperienceReady` works in demo_game; integration test missing |
| P1-07 | PARTIAL | `PendingInit` cleared in init systems; integration test missing |
| P1-08 | PASS | 4 abilities in RON + `AbilityImplRegistry` in demo_game |
| P1-09 | FAIL | `bench_100_asc` missing |
| P1-10 | PARTIAL | Playable 10-min loop locally; formal gate not recorded |

**GATE: FAIL**

## Gate P2 - Multiplayer Streamed World

| ID | Status | Evidence |
|----|--------|----------|
| P2-01 | FAIL | `dm_8p_lan` playtest missing |
| P2-02 | FAIL | Dedicated server binary missing |
| P2-03 | FAIL | Health replication test missing |
| P2-04 | FAIL | Net bandwidth overlay/artifact missing |
| P2-05 | FAIL | `sector_walk` playtest missing |
| P2-06 | FAIL | Sector perf benchmark missing |
| P2-07 | FAIL | Relevancy test missing |
| P2-08 | FAIL | Prediction playtest missing |
| P2-09 | FAIL | PlayerState persistence net test missing |
| P2-10 | FAIL | `examples/lyra_equivalent` missing |

**GATE: FAIL**

## Gate P3 - Studio / Agent

| ID | Status | Evidence |
|----|--------|----------|
| P3-01 | FAIL | Project-local `aa validate` missing |
| P3-02 | FAIL | Bootstrap `./aa validate examples/demo_game --format sarif` emits schema-checked SARIF 2.1-shaped diagnostics; Rust `aa validate --format sarif` missing |
| P3-03 | FAIL | Bootstrap `./aa playtest --scenario smoke --json` exits 0 with schema-valid fixture output; runtime playtest CI is still missing |
| P3-04 | FAIL | Bootstrap scene list/inspect and patch dry-run report authored-sector context, affected files/entities, and undo token without mutation; runtime Scene RPC, patch apply, and real undo implementation missing |
| P3-05 | FAIL | Bootstrap `demo_game_add_fire_ability` eval passes; runtime eval harness and Rust `aa eval run` missing |
| P3-06 | FAIL | Editor save path missing |
| P3-07 | FAIL | Hot reload test missing |
| P3-08 | FAIL | 20 playtest scenarios missing |

**GATE: FAIL**

## Gate P4 - Presentation AA

| ID | Status | Evidence |
|----|--------|----------|
| P4-01 | FAIL | Scalability preset proof missing |
| P4-02 | FAIL | Post stack sign-off missing |
| P4-03 | FAIL | Motion matching playtest/sign-off missing |
| P4-04 | FAIL | Virtual geometry scene proof missing |
| P4-05 | FAIL | Tracy frame budget artifact missing |
| P4-06 | FAIL | Material graph editor proof missing |

**GATE: FAIL**

## Gate AA - Product Certification

| ID | Status | Evidence |
|----|--------|----------|
| AA-01 | FAIL | `examples/lyra_equivalent` missing |
| AA-02 | FAIL | Automated REQ coverage evidence missing |
| AA-03 | FAIL | `aa_*` crate coverage evidence missing |
| AA-04 | UNKNOWN | Bug tracker not available in this checkout |
| AA-05 | PARTIAL | Public docs and `AGENTS.md` exist; implemented product missing |
| AA-06 | FAIL | 8-player public playtest evidence missing |

**GATE: FAIL**

## Gate OWA - Open World Alpha

| ID | Status | Evidence |
|----|--------|----------|
| OWA-01 | FAIL | Checkable `examples/open_world_studio` contract package exists, but the required 16 km2 authored runtime world is not implemented |
| OWA-02 | FAIL | Runtime `aa world inspect` missing; bootstrap fixture bridge exists |
| OWA-03 | FAIL | Runtime `aa world inspect` missing; bootstrap fixture bridge exists |
| OWA-04 | FAIL | Runtime `open_world_sector_traverse` playtest missing; bootstrap enemy-camp playtest result fixture exists |
| OWA-05 | FAIL | Runtime streaming profile artifact missing; bootstrap profile fixture bridge exists |
| OWA-06 | FAIL | Runtime `aa world cook --verify` missing; bootstrap cook fixture bridge exists |
| OWA-07 | FAIL | `add_enemy_camp` runtime eval missing; spec fixture exists |
| OWA-08 | FAIL | `add_elemental_ability` eval missing |

**GATE: FAIL**

## Gate OWB - Open World Beta

| ID | Status | Evidence |
|----|--------|----------|
| OWB-01 | FAIL | 64 km2 reference world missing |
| OWB-02 | FAIL | Content-scale world inspection missing |
| OWB-03 | FAIL | 64 km2 streaming profile missing |
| OWB-04 | FAIL | Crowd LOD profile/playtest missing |
| OWB-05 | FAIL | `open_world_32p` net playtest missing |
| OWB-06 | FAIL | `world_delta_persist` playtest missing |
| OWB-07 | FAIL | 100k asset validation benchmark missing |
| OWB-08 | FAIL | Passing agent eval suite report missing; bootstrap eval report exists and fails honestly for missing implementation |

**GATE: FAIL**

## Gate AS - AAA Studio

| ID | Status | Evidence |
|----|--------|----------|
| AS-01 | FAIL | Runtime prompt-to-feature eval suite missing; bootstrap eval report bridge exists for `open_world_studio_enemy_camp` |
| AS-02 | FAIL | Repair-attempt eval report missing |
| AS-03 | FAIL | End-to-end eval artifacts missing |
| AS-04 | FAIL | Human acceptability review sample missing |
| AS-05 | FAIL | `editor_cli_patch_parity` integration test missing |
| AS-06 | FAIL | 64 km2 `add_enemy_camp` eval missing |

**GATE: FAIL**

## Next Gate Attempt

The next meaningful gate attempt is P0. The shortest path is:

1. Scaffold project-local `crates/aa_cli` after human approval to add workspace crates.
2. Port `./aa validate` into the Rust `aa validate --format json`.
3. Port `./aa index` into the Rust `aa index --query`.
4. Port `./aa check` into the Rust `aa check --json`.
5. Port `./aa ability graph` into the Rust `aa ability graph <id> --json`.
6. Port `./aa config get`, `./aa eval list`, and `./aa eval run` into the Rust CLI.
7. Turn `examples/demo_game` contract assets into a runtime combat smoke path.
8. Record command outputs here.

Open-world CLI work can begin from the existing `world_inspect_result`, `world_cook_result`, `world_generate_result`, `config_get_result`, `eval_list_result`, and `profile_summary_result` schemas plus `docs/specs/fixtures/open_world_studio/` once the P0 CLI exists.
