# 02 — Performance Budgets

> **Normative.** Measured on reference hardware unless noted.

## Reference Hardware

| Profile | CPU | GPU | RAM | OS |
|---------|-----|-----|-----|-----|
| **Dev** | Apple M2 / Ryzen 5 5600 | Integrated or GTX 1660 | 16 GB | macOS / Windows |
| **AA Target** | Ryzen 5 7600 | RTX 3060 | 16 GB | Windows 11 |
| **Server** | 4 vCPU | None | 8 GB | Linux x64 |

## Frame Budget (60 FPS = 16.67ms total)

| Subsystem | Budget (ms) | @ condition | Measurement |
|-----------|-------------|-------------|-------------|
| **Total frame** | ≤ 16.67 | AA target HW, 1080p High | Tracy `frame` |
| Input + mapping | ≤ 0.2 | 4 local players | `aa_input` span |
| Ability tick (all) | ≤ 0.5 | 100 ASCs, 50 active effects | `aa_ability` span |
| Character movement | ≤ 0.8 | 16 characters | `aa_physics` span |
| Rapier step | ≤ 2.0 | 2k rigid bodies, 16 characters | physics step |
| Animation eval | ≤ 2.0 | 16 skeletal meshes | `aa_animation` span |
| Crowd processors | ≤ 1.0 | 1,000 agents | `aa_crowd` span |
| World stream policy | ≤ 0.3 | 5×5 active sectors | `aa_world_stream` |
| Net receive + send | ≤ 2.0 | 8 players, 128 kbps cap | `aa_net` span |
| Render (Bevy) | ≤ 10.0 | 1080p High preset | `bevy_render` |
| Headroom | ≥ 2.0 | — | — |

## Memory Budgets

| Resource | Budget | @ condition |
|----------|--------|-------------|
| **Total process RSS** | ≤ 4 GB | Client AA target, streamed world |
| **Server RSS** | ≤ 2 GB | 16 players, no render |
| Active sector data | ≤ 512 MB | 25 active sectors |
| Asset registry index | ≤ 64 MB | 10,000 assets |
| DDC cache (disk) | ≤ 20 GB | dev machine |
| Net snapshot buffers | ≤ 32 MB | 16 players rewind |

## Network Budgets

| Metric | Budget | @ condition |
|--------|--------|-------------|
| Tick rate | 60 Hz | server simulation |
| Snapshot rate | 20–60 Hz | per-entity class configurable |
| Bandwidth per client | ≤ 128 kbps p95 | 8 players, combat |
| RTT playable | ≤ 150 ms | with prediction |
| Correction magnitude | ≤ 0.5m p95 | @ 100ms RTT, locomotion |
| Join time | ≤ 10 s | experience + 9 sectors loaded |

## Streaming Budgets

| Metric | Budget |
|--------|--------|
| Sector async load p95 | ≤ 500 ms |
| Sector activations per frame | ≤ 2 |
| Sector deactivations per frame | ≤ 4 |
| Hitch on sector crossing | ≤ 8 ms frame spike |
| Navmesh stitch on load | ≤ 100 ms background |

## Tooling Budgets

| Command | Budget |
|---------|--------|
| `aa check` | ≤ 120 s cold, ≤ 30 s incremental |
| `aa validate` (1000 assets) | ≤ 10 s |
| `aa index` full rebuild | ≤ 60 s |
| `aa playtest` smoke (30s) | ≤ 45 s wall clock |
| RON hot reload | ≤ 500 ms perceived |

## Scalability Presets (1080p)

| Preset | Resolution scale | Shadow res | GI | Target FPS |
|--------|------------------|------------|-----|------------|
| Low | 0.5 | 1024 | Off | 60 |
| Medium | 0.75 | 2048 | Baked | 60 |
| High | 1.0 | 4096 | Probes | 60 |
| Epic | 1.0 | 4096 | Probes+ | 60 (30 on dev GPU) |

## Benchmark Requirements

Each crate MUST ship `benches/` or integration perf tests for its budget rows.

| Crate | Benchmark |
|-------|-----------|
| `aa_ability` | `bench_100_asc_effects` |
| `aa_net` | `bench_8_client_replication` |
| `aa_world_stream` | `bench_sector_crossing` |
| `aa_animation` | `bench_16_skeletons` |

CI MUST fail if budget regresses > 10% from baseline on `AA Target` profile (weekly job).

## Measurement Tools

| Tool | Use |
|------|-----|
| Tracy | Frame + span |
| `criterion` | Microbenches |
| `aa playtest --profile` | End-to-end |
| Net overlay | Bytes/s per connection |
