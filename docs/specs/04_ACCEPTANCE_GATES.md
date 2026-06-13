# 04 — Acceptance Gates

> **Normative.** Measurable exit criteria per phase. Gate is PASS only when ALL rows green.

---

## Gate P0 — Foundation

**Target:** Week 6 | **Blocks:** all other gates

| ID | Criterion | Measurement | PASS threshold |
|----|-----------|-------------|----------------|
| P0-01 | Workspace compiles | `cargo check --workspace` | exit 0 |
| P0-02 | Config merge | test `config_merge_order` | all REQ-GLOBAL-050 cases pass |
| P0-03 | Prefab spawn | integration `spawn_player_prefab` | entity has ≥ 3 components |
| P0-04 | Schedule order | test `schedule_ambiguity` | 0 ambiguities |
| P0-05 | Validate stub | `aa validate` on empty project | exit 0 |
| P0-06 | Platform boot | manual | window opens Win + macOS ≤ 30s from cold |
| P0-07 | Spec traceability | audit | ≥ 50 REQ-* mapped to tests |

**Artifacts:** `examples/demo_game` runs, `aa_cli check` works.

---

## Gate P1 — Combat Vertical Slice

**Target:** Month 6 | **Blocks:** P2, P3 agent gameplay tasks

| ID | Criterion | Measurement | PASS threshold |
|----|-----------|-------------|----------------|
| P1-01 | Locomotion | playtest `locomotion_smoke` | 60s no fall-through-world |
| P1-02 | Fire ability | playtest `fireball_hit` | target Health −25 server-side |
| P1-03 | Effect via GAS | unit `effect_modifies_attribute` | only via GameplayEffect |
| P1-04 | ASC placement | unit `asc_on_player_state` | survives pawn despawn |
| P1-05 | Tags block ability | unit `stun_blocks_fire` | activation denied |
| P1-06 | Experience boot | integration `experience_load` | `ExperienceReady` < 5s |
| P1-07 | Init FSM | integration `pawn_init_chain` | `PendingInit` removed < 10 frames |
| P1-08 | Data-driven | audit | ≥ 3 abilities in RON only |
| P1-09 | Ability perf | bench `bench_100_asc` | ≤ 0.5ms |
| P1-10 | Playable demo | human | 10-min loop: move, fire, die, respawn |

**Artifacts:** `examples/demo_game` combat playable single-player.

---

## Gate P2 — Multiplayer Streamed World

**Target:** Month 14 | **Blocks:** AA certification

| ID | Criterion | Measurement | PASS threshold |
|----|-----------|-------------|----------------|
| P2-01 | 8-player LAN | playtest `dm_8p_lan` | 5 min, 0 disconnects |
| P2-02 | Dedicated server | `aa_server` headless | 8 clients connect |
| P2-03 | Health replicate | net test | client sees server Health ±0.1 |
| P2-04 | Bandwidth | net overlay | p95 ≤ 128 kbps per client |
| P2-05 | Sector cross | playtest `sector_walk` | 0 crashes, 0 orphan entities |
| P2-06 | Sector perf | bench | p95 load ≤ 500ms |
| P2-07 | Relevancy | net test | distant pawn not replicated |
| P2-08 | Prediction | playtest @ 100ms RTT | playable, correction p95 ≤ 0.5m |
| P2-09 | PlayerState persist | net test | abilities survive respawn |
| P2-10 | Reference game | `examples/lyra_equivalent` | P2 playtests all green |

**Artifacts:** dedicated server binary, 9-sector arena.

---

## Gate P3 — Studio / Agent

**Target:** Month 18 | **Blocks:** public studio beta

| ID | Criterion | Measurement | PASS threshold |
|----|-----------|-------------|----------------|
| P3-01 | Validate speed | `aa validate` 1000 assets | ≤ 10s |
| P3-02 | SARIF output | validator test | schema-valid SARIF 2.1 |
| P3-03 | Playtest CI | `playtest smoke` in CI | ≤ 45s |
| P3-04 | Scene RPC | integration `scene_patch` | round-trip + undo |
| P3-05 | Agent task | eval `add_fire_ability` | pass without human edit |
| P3-06 | Editor save | manual | scene RON round-trip lossless |
| P3-07 | Hot reload | test | RON effect reload ≤ 500ms |
| P3-08 | Scenario count | audit | ≥ 20 playtest scenarios |

**Artifacts:** JSON-RPC server, `AGENTS.md` enforced in CI.

---

## Gate P4 — Presentation AA

**Target:** Month 28 | **Blocks:** AA visual label

| ID | Criterion | Measurement | PASS threshold |
|----|-----------|-------------|----------------|
| P4-01 | Scalability | manual | 4 presets change FPS ±30% |
| P4-02 | Post stack | visual | bloom + tonemap + AA |
| P4-03 | Motion matching | playtest | locomotion blend quality sign-off |
| P4-04 | Virtual geometry | scene | static env uses VG path |
| P4-05 | Frame budget | Tracy | ≤ 16.67ms @ AA Target High |
| P4-06 | Material graph | editor | 1 custom PBR material ships |

---

## Gate AA — Product Certification

**All of:** P0 + P1 + P2 + P3 MUST pass. P4 MUST pass for "AA Visual" label.

| ID | Criterion | PASS |
|----|-----------|------|
| AA-01 | `examples/lyra_equivalent` complete | REQUIRED |
| AA-02 | REQ coverage ≥ 95% automated | REQUIRED |
| AA-03 | Line coverage `aa_*` ≥ 80% | REQUIRED |
| AA-04 | No P0/P1 open severity bugs | REQUIRED |
| AA-05 | Public docs + AGENTS.md | REQUIRED |
| AA-06 | 8-player public playtest session | REQUIRED |

---

## Gate OWA — Open World Alpha

**Target:** Post-AA | **Blocks:** AAA/Open World claim

| ID | Criterion | Measurement | PASS threshold |
|----|-----------|-------------|----------------|
| OWA-01 | Reference open-world game | `examples/open_world_studio` | 16 km2 authored world |
| OWA-02 | Sector count | `aa world inspect` | ≥ 64 authored sectors |
| OWA-03 | Data layers | `aa world inspect` | ≥ 8 named layers |
| OWA-04 | Sector traversal | playtest `open_world_sector_traverse` | 10 min, 0 crashes, 0 orphan state |
| OWA-05 | Streaming perf | profile `open_world_traverse` | p95 load ≤ 400ms, hitch ≤ 6ms |
| OWA-06 | Cook artifacts | `aa world cook --verify` | render, physics, nav, spawn metadata present |
| OWA-07 | Agent camp task | eval `add_enemy_camp` | pass without human edit |
| OWA-08 | Agent ability task | eval `add_elemental_ability` | pass without human edit |

**Artifacts:** cooked sector artifacts, profile trace, eval logs, reference game.

---

## Gate OWB — Open World Beta

**Target:** Post-OWA | **Blocks:** "AAA-capable open world" label

| ID | Criterion | Measurement | PASS threshold |
|----|-----------|-------------|----------------|
| OWB-01 | World scale | `examples/open_world_studio` | 64 km2 authored world |
| OWB-02 | Content scale | `aa world inspect` | ≥ 1M authored placed objects via sectors/HLOD |
| OWB-03 | Streaming perf | profile `open_world_traverse_64km2` | p95 load ≤ 300ms, hitch ≤ 4ms |
| OWB-04 | Crowd scale | playtest/profile `crowd_lod_open_world` | 1,000 full AI + 10,000 low-LOD agents |
| OWB-05 | Multiplayer relevancy | net playtest `open_world_32p` | 32 players, no distant broadcast replication |
| OWB-06 | Save/load deltas | playtest `world_delta_persist` | player/world deltas survive reload |
| OWB-07 | Validate scale | `aa validate` on 100k assets | ≤ 60s |
| OWB-08 | Agent repair loop | eval suite | ≥ 80% pass, ≤ 3 repair attempts |

**Artifacts:** 64 km2 reference build, network traces, validation benchmark, eval report.

---

## Gate AS — AAA Studio

**Target:** Post-OWB | **Blocks:** "Cursor for games" claim

| ID | Criterion | Measurement | PASS threshold |
|----|-----------|-------------|----------------|
| AS-01 | Prompt-to-feature suite | eval suite in `06_AAA_OPEN_WORLD_STUDIO.md` | ≥ 90% pass |
| AS-02 | Repair attempts | eval report | ≤ 2 repair attempts average |
| AS-03 | End-to-end proof | selected eval artifacts | index, patch, validate, check, graph, playtest all recorded |
| AS-04 | Human acceptability | review sample | ≥ 80% generated diffs accepted without manual rewrite |
| AS-05 | Shared patch backend | integration `editor_cli_patch_parity` | CLI and editor produce same undoable patch result |
| AS-06 | Open-world feature task | eval `add_enemy_camp` on 64 km2 world | pass without human edit |

**Artifacts:** eval corpus, machine-readable reports, patch logs, playtest/profile artifacts.

## Gate Status Template

```markdown
## Gate P1 — Status
- Date: YYYY-MM-DD
- Commit: abc123
- Owner: @name
- P1-01: PASS (link CI run)
- P1-02: FAIL (issue #123)
...
**GATE: FAIL**
```

Record gate status in `docs/specs/GATE_STATUS.md` (create on first gate attempt).
