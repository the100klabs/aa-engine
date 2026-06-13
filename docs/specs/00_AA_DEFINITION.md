# 00 — AA Definition

> **Normative.** This document defines what "AA" means for the AA Engine and Cursor-like Game Studio product.

## Product Statement

**AA Engine** is an open-source, ECS-native gameplay framework on Bevy that enables **AA-quality indie games** (Palworld / Helldivers 2 / Remnant-scale systems complexity, not film-tier UE5 visuals) built through a **Cursor-like agent studio** where chat drives asset/code edits validated by automated playtests.

## AA Tier Definitions

| Tier | Code | Definition | Timeline |
|------|------|------------|----------|
| **Research** | R | UE5 analysis, design sketches | Done (`docs/research/`) |
| **MVP** | M | Single-player vertical slice, dev tooling stubs | 3–6 months |
| **AA** | A | Shippable indie AA per this document | 18–36 months |
| **UE5 Parity** | U | Not a goal | ∞ |

**This spec set targets tier A.**

## Post-AA Ambition

The larger "AAA open world engine + Cursor-like studio" goal is tracked separately in
[`06_AAA_OPEN_WORLD_STUDIO.md`](./06_AAA_OPEN_WORLD_STUDIO.md). That track starts after
the AA gates are real and measurable. It does not change the AA definition, and it does
not make UE5 parity a goal.

## AA Visual Tier (Rendering)

| Capability | AA requirement | UE5 reference | Non-goal |
|------------|----------------|---------------|----------|
| PBR materials | MUST | Standard deferred/forward | Substrate film pipeline |
| Shadow quality | High-res CSM + local lights | VSM concept | Full VSM parity |
| GI | Probe grid OR baked + SSAO | Lumen | Dynamic Lumen parity |
| Geometry | LOD + optional virtual geometry | Nanite | Film Nanite parity |
| Characters | Skeletal + blend spaces + IK foot | AnimBP subset | Motion matching AAA |
| VFX | GPU particles + graph subset | Niagara subset | Full Niagara |

## AA Gameplay Tier

| Capability | AA requirement | UE5 reference |
|------------|----------------|---------------|
| Ability system | Full GAS subset: attributes, effects, tags, cues, cooldowns, costs | GAS plugin |
| Damage | 100% via GameplayEffects | Epic internal policy |
| Player persistence | ASC on PlayerState | GAS README |
| Experiences | Data-driven match boot | Lyra ExperienceDefinition |
| Game features | Compile-time feature crates | Lyra GameFeatures |
| Input | Semantic actions + mapping contexts | Enhanced Input |
| Init states | Ordered pawn boot FSM | Lyra InitStateInterface |

## AA Multiplayer Tier

| Metric | AA requirement |
|--------|----------------|
| Players per match | 8–16 (AA), 32 (stretch) |
| Tick rate | 60 Hz simulation |
| RTT tolerance | Playable at 150ms RTT |
| Bandwidth per client | ≤ 128 kbps @ 8 players |
| Dedicated server | MUST (Linux x64) |
| Prediction | Character movement + ability activation (owning client) |
| Relevancy | Spatial graph + always-relevant sets |

## AA World Tier

| Metric | AA requirement |
|--------|----------------|
| World size | 4–16 km² streamed |
| Sector size | 256m default |
| Active window | 5×5 sectors per source |
| Load budget | ≤ 2 sector activations/frame |
| Load latency | p95 < 500ms async |
| Crowd agents | 1,000 @ 60fps (ISM), 10,000 (low LOD) |
| Data layers | ≥ 8 named layers |

## AA Studio Tier (Differentiator)

| Capability | AA requirement |
|------------|----------------|
| `aa validate` | Full project < 10s @ 1000 assets |
| `aa playtest` | Scenario library ≥ 20 tests |
| Agent repair loop | ≥ 70% compile-error auto-fix in controlled eval |
| Scene patch API | JSON-RPC with undo tokens |
| Text-first assets | ≥ 90% gameplay data in RON/TOML |
| Hot reload | Ability/effect RON reload < 500ms |

## AA Engineering Tier

| Metric | AA requirement |
|--------|----------------|
| Test coverage (aa_* crates) | ≥ 80% line coverage |
| CI time | < 15 min full pipeline |
| Spec traceability | 100% REQ-* mapped to test |
| Breaking changes | Semver + migration guide |
| Documentation | Every public API has rustdoc + SPEC cross-ref |

## Explicit Non-Goals (AA scope boundary)

| Non-goal | Reason |
|----------|--------|
| Blueprint visual scripting | Agent + RON replaces |
| Nanite/Lumen production parity | R&D scale |
| Console certification | Post-AA |
| MetaHuman / facial animation | Out of scope |
| Full Sequencer film pipeline | AA gets camera tracks only |
| Marketplace / FAB | Post-AA |
| Mobile P0 | Desktop first |

## Reference Game Requirement

AA tier MUST include a working reference game:

**`examples/lyra_equivalent/`** — not a Lyra port, a proof of:

- Experience-driven match boot
- 8-player networked shooter DM
- 3 abilities (fire, dash, melee)
- Streamed arena map (≥ 9 sectors)
- Agent-completable task: "add new GameplayEffect"

No AA claim without this example passing all P2 acceptance gates in `04_ACCEPTANCE_GATES.md`.

## Sign-Off Authority

A subsystem is **AA-certified** when:

1. All `REQ-{CRATE}-*` in its SPEC are implemented
2. All SPEC test matrix rows are automated green in CI
3. Performance budgets in `02_PERFORMANCE_BUDGETS.md` verified by benchmark
4. Entry added in `04_ACCEPTANCE_GATES.md` with date + commit SHA
