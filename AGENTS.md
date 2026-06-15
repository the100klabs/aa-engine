# AGENTS.md — Rules for AI Assistants

> **Normative specs:** `docs/specs/` — implement from `aa_*/SPEC.md`, not research docs.

> Place this file at the **root of every game project** created with the AA engine stack.

## Project Identity

This is an **AA Engine** project: Bevy ECS runtime inspired by UE5 *responsibilities*, not implementation. Architecture reference: `docs/research/unreal_to_bevy/`.

## Before Any Edit

1. Read `aa.project.toml` for project structure
2. Run `aa index --query "<relevant topic>"` if available
3. Check `16_anti_patterns_and_decisions.md` for forbidden patterns

## Mandatory Verification Commands

| After editing | Run |
|---------------|-----|
| Rust code | `aa check` |
| RON/TOML assets | `aa validate` |
| Scene/prefab | `aa validate` + `aa scene inspect` if entity work |
| Abilities/effects | `aa ability graph <ability_id>` |
| Claiming task complete | `aa playtest --scenario smoke` |

## Architecture Rules (non-negotiable)

### ECS
- Use components + systems, never `struct Actor { ... }`
- Use `Possesses` / `PossessedBy` relationships for controller↔pawn
- Query with `Without<PendingInit>` for gameplay systems on pawns

### Abilities (GAS-inspired)
- ASC on **PlayerState** for human players, not Pawn
- All attribute changes via `GameplayEffect` — never direct writes
- Register new tags in `assets/data/tags.ron`
- New abilities need: `.ron` asset + Rust registrar in `aa_ability`

### Assets
- All gameplay data in RON under `assets/`
- Every asset needs `schema_version: 1` (or current)
- Soft refs are string paths, not file system absolute paths
- Binary meshes in `assets/meshes/`, not embedded in RON

### Networking (when enabled)
- Gate server logic with `AppRole::DedicatedServer | ListenServer`
- Replicate only registered components per `config/replication.toml`
- No replicated UI state

### Schedules
- Input → `PreUpdate`
- Movement intent → `FixedPrePhysics`
- Physics → `FixedUpdate`
- Animation → `Update`
- Net send → `PostUpdate`
- See `14_system_schedule_spec.md`

## File Placement Guide

| What | Where |
|------|-------|
| New ability | `assets/abilities/<name>.ron` + `crates/aa_ability/src/abilities/<name>.rs` |
| New effect | `assets/effects/<name>.ron` |
| New pawn type | `assets/pawns/<name>.ron` |
| Input bindings | `assets/input/contexts/<name>.ron` |
| Experience/mode | `assets/experiences/<name>.ron` |
| System code | `crates/aa_*/src/` |
| Game-specific logic | `src/` or `examples/<game>/src/` |

## Do NOT

- Copy or translate Unreal Engine C++ source
- Create Blueprint-style visual scripting unless explicitly requested
- Put gameplay tuning in Rust when RON assets suffice
- Skip validation before saying "done"
- Edit `target/`, `Cargo.lock` without user request
- Invent schema fields not in `13_data_schemas.md` without updating that doc

## Preferred Workflow for Features

```
1. Design: which crates touched? (see 12_integration_blueprint.md)
2. Schema: add/update RON asset types if needed
3. Data: create RON assets with valid refs
4. Code: minimal systems + registrar
5. Validate: aa validate + aa check
6. Playtest: relevant scenario
```

## Lyra-Equivalent Patterns

| Need | Pattern |
|------|---------|
| Match setup | `ExperienceDefinition` asset |
| Mode plugins | `feature_*` Cargo crates |
| Pawn setup | `PawnData` + init state machine |
| Combat | `aa_ability` + tags |

## Ask Human When

- Breaking `schema_version`
- Adding new crate to workspace
- Changing replication manifest
- Network prediction scope changes
- Rendering pipeline modifications

## Reference Docs (read order)

1. **`docs/specs/README.md`** — normative AA spec index (BUILD FROM THIS)
2. `docs/specs/04_ACCEPTANCE_GATES.md` — measurable pass/fail
3. `docs/specs/aa_cli/SPEC.md` — agent CLI contract
4. `docs/research/unreal_to_bevy/12_integration_blueprint.md` — wiring (informative)
5. Research `01`–`11` — UE5 rationale only

## Success Criteria

A task is complete when:
- [ ] `aa check` exits 0
- [ ] `aa validate` exits 0
- [ ] Relevant playtest scenario passes
- [ ] No anti-patterns from doc 16 introduced
- [ ] Schema changes reflected in `13_data_schemas.md`

## Cursor Cloud specific instructions

Standard commands live in `README.md` and `.github/workflows/ci.yml`; this section only captures non-obvious caveats for this environment.

### Toolchain
- The workspace is edition 2024 / `rust-version = "1.85.0"` (`Cargo.toml`). The VM base image may ship an older Rust (e.g. 1.83) that **cannot** compile this repo; use `rustup default stable` (≥1.85). The startup update script keeps `cargo` deps fetched.
- Python tooling (`docs/specs/tools/*.py`, run via `./aa <non-rust-cmd>`) is stdlib-only on Python 3.12 — no pip install needed.

### Running the game / playtests is GPU-gated (key gotcha)
- `cargo run -p demo_game` and `aa playtest` boot the full Bevy renderer, which **panics with "Unable to find a GPU!"** in this headless VM unless a Vulkan driver is present. The Mesa software driver (lavapipe / `mesa-vulkan-drivers`) is installed in the VM image.
- You MUST point Vulkan at lavapipe and give it a runtime dir before running any renderer/playtest command:
  ```bash
  export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json
  export XDG_RUNTIME_DIR=/tmp/xdg && mkdir -p /tmp/xdg
  ```
  Without these, the renderer cannot find an adapter. With them, the log shows `AdapterInfo { ... driver: "llvmpipe" ... }` and the app runs (slowly — software rendering).
- "No audio device found" / ALSA `cannot find card '0'` warnings are expected and harmless (no sound hardware).

### Known pre-existing failures (NOT environment issues)
- `cargo run -p aa_cli -- validate examples/demo_game` currently exits 1 with `SCHEMA_INVALID: $.cue_on_activate ... got array` on the ability RONs. This also fails CI on `main` (red), so treat it as a pre-existing content/schema mismatch, not a setup problem.
- Combat playtests (e.g. `smoke`, `fireball_hit`) currently fail their `dummy_damaged` assertion (pre-existing). A clean end-to-end smoke that passes today is `aa playtest --project examples/demo_game --scenario locomotion_smoke --duration 12`.
