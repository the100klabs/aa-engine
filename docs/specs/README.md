# AA Engine — Implementation Specifications

> **Normative.** These documents are the build contract. Research docs in `docs/research/unreal_to_bevy/` are appendix only.

## Document Hierarchy

| Tier | Location | Status | Use |
|------|----------|--------|-----|
| **AA Spec** | `docs/specs/` (this folder) | **Normative** | Implement + test against |
| **Research** | `docs/research/unreal_to_bevy/` | Informative | UE5 analysis, design rationale |

If research and spec conflict, **spec wins**.

## Spec Index

| Doc | Purpose |
|-----|---------|
| [00_AA_DEFINITION.md](./00_AA_DEFINITION.md) | What "AA" means for this product |
| [01_GLOBAL_REQUIREMENTS.md](./01_GLOBAL_REQUIREMENTS.md) | Cross-cutting MUST/SHALL requirements |
| [02_PERFORMANCE_BUDGETS.md](./02_PERFORMANCE_BUDGETS.md) | Per-subsystem CPU/memory/net budgets |
| [03_PLATFORM_MATRIX.md](./03_PLATFORM_MATRIX.md) | Platform support tiers |
| [04_ACCEPTANCE_GATES.md](./04_ACCEPTANCE_GATES.md) | Phase exit gates (measurable) |
| [05_TRACEABILITY_MATRIX.md](./05_TRACEABILITY_MATRIX.md) | UE5 → REQ → crate mapping |
| [06_AAA_OPEN_WORLD_STUDIO.md](./06_AAA_OPEN_WORLD_STUDIO.md) | Post-AA AAA open-world and Cursor-like studio track |
| [07_ENGINE_STUDIO_EXECUTION_PLAN.md](./07_ENGINE_STUDIO_EXECUTION_PLAN.md) | Ordered implementation backlog for engine + studio |
| [GATE_STATUS.md](./GATE_STATUS.md) | Current evidence snapshot for acceptance gates |

### Subsystem Spec Packages

| Crate | Spec | Priority |
|-------|------|----------|
| `aa_core` | [aa_core/SPEC.md](./aa_core/SPEC.md) | P0 |
| `aa_assets` | [aa_assets/SPEC.md](./aa_assets/SPEC.md) | P0 |
| `aa_scene` | [aa_scene/SPEC.md](./aa_scene/SPEC.md) | P0 |
| `aa_tags` | [aa_tags/SPEC.md](./aa_tags/SPEC.md) | P1 |
| `aa_input` | [aa_input/SPEC.md](./aa_input/SPEC.md) | P1 |
| `aa_ability` | [aa_ability/SPEC.md](./aa_ability/SPEC.md) | P1 |
| `aa_gameplay` | [aa_gameplay/SPEC.md](./aa_gameplay/SPEC.md) | P1 |
| `aa_experience` | [aa_experience/SPEC.md](./aa_experience/SPEC.md) | P1 |
| `aa_physics` | [aa_physics/SPEC.md](./aa_physics/SPEC.md) | P1 |
| `aa_animation` | [aa_animation/SPEC.md](./aa_animation/SPEC.md) | P1 |
| `aa_net` | [aa_net/SPEC.md](./aa_net/SPEC.md) | P2 |
| `aa_world_stream` | [aa_world_stream/SPEC.md](./aa_world_stream/SPEC.md) | P2 |
| `aa_cli` | [aa_cli/SPEC.md](./aa_cli/SPEC.md) | P0 (studio wedge) |
| `aa_editor` | [aa_editor/SPEC.md](./aa_editor/SPEC.md) | P3 |
| `aa_render` | [aa_render/SPEC.md](./aa_render/SPEC.md) | P4 |

### Formal Schemas

| Schema | File |
|--------|------|
| Project manifest | [schemas/project.schema.json](./schemas/project.schema.json) |
| Asset manifest | [schemas/asset_manifest.schema.json](./schemas/asset_manifest.schema.json) |
| Engine config | [schemas/config_engine.schema.json](./schemas/config_engine.schema.json) |
| Game config | [schemas/config_game.schema.json](./schemas/config_game.schema.json) |
| Input config | [schemas/config_input.schema.json](./schemas/config_input.schema.json) |
| Scalability config | [schemas/config_scalability.schema.json](./schemas/config_scalability.schema.json) |
| Tag dictionary | [schemas/tag_dictionary.schema.json](./schemas/tag_dictionary.schema.json) |
| Gameplay ability | [schemas/gameplay_ability.schema.json](./schemas/gameplay_ability.schema.json) |
| Gameplay effect | [schemas/gameplay_effect.schema.json](./schemas/gameplay_effect.schema.json) |
| Attribute set | [schemas/attribute_set.schema.json](./schemas/attribute_set.schema.json) |
| Pawn data | [schemas/pawn_data.schema.json](./schemas/pawn_data.schema.json) |
| AI profile | [schemas/ai_profile.schema.json](./schemas/ai_profile.schema.json) |
| Spawn table | [schemas/spawn_table.schema.json](./schemas/spawn_table.schema.json) |
| Input context | [schemas/input_context.schema.json](./schemas/input_context.schema.json) |
| Experience | [schemas/experience.schema.json](./schemas/experience.schema.json) |
| Action set | [schemas/action_set.schema.json](./schemas/action_set.schema.json) |
| Prefab | [schemas/prefab.schema.json](./schemas/prefab.schema.json) |
| World | [schemas/world.schema.json](./schemas/world.schema.json) |
| Sector | [schemas/sector.schema.json](./schemas/sector.schema.json) |
| Scene patch | [schemas/scene_patch.schema.json](./schemas/scene_patch.schema.json) |
| Scene list result | [schemas/scene_list_result.schema.json](./schemas/scene_list_result.schema.json) |
| Scene inspect result | [schemas/scene_inspect_result.schema.json](./schemas/scene_inspect_result.schema.json) |
| Scene patch result | [schemas/scene_patch_result.schema.json](./schemas/scene_patch_result.schema.json) |
| Playtest scenario | [schemas/playtest_scenario.schema.json](./schemas/playtest_scenario.schema.json) |
| Playtest result | [schemas/playtest_result.schema.json](./schemas/playtest_result.schema.json) |
| Ability graph result | [schemas/ability_graph_result.schema.json](./schemas/ability_graph_result.schema.json) |
| Agent eval suite | [schemas/agent_eval.schema.json](./schemas/agent_eval.schema.json) |
| Eval list result | [schemas/eval_list_result.schema.json](./schemas/eval_list_result.schema.json) |
| Eval report | [schemas/eval_report.schema.json](./schemas/eval_report.schema.json) |
| Config get result | [schemas/config_get_result.schema.json](./schemas/config_get_result.schema.json) |
| Index query result | [schemas/index_result.schema.json](./schemas/index_result.schema.json) |
| Validation result | [schemas/validation_result.schema.json](./schemas/validation_result.schema.json) |
| SARIF 2.1 minimum validation result | [schemas/sarif_2_1_min.schema.json](./schemas/sarif_2_1_min.schema.json) |
| Check result | [schemas/check_result.schema.json](./schemas/check_result.schema.json) |
| World inspect result | [schemas/world_inspect_result.schema.json](./schemas/world_inspect_result.schema.json) |
| World cook result | [schemas/world_cook_result.schema.json](./schemas/world_cook_result.schema.json) |
| World generate result | [schemas/world_generate_result.schema.json](./schemas/world_generate_result.schema.json) |
| Profile summary result | [schemas/profile_summary_result.schema.json](./schemas/profile_summary_result.schema.json) |

### Contract Fixtures

| Fixture set | Purpose |
|-------------|---------|
| [fixtures/empty_project](./fixtures/empty_project) | Minimal initialized AA project with no gameplay assets; `./aa validate docs/specs/fixtures/empty_project --format json` exits 0 |
| [fixtures/demo_game](./fixtures/demo_game) | Golden examples for the first Cursor-like studio slice: add a fire ability, inspect the ability graph, and pass `fireball_hit` |
| [fixtures/open_world_studio](./fixtures/open_world_studio) | Golden examples for the first AAA/Open World studio slice: add an enemy camp, inspect/cook the world, playtest traversal/combat, and summarize performance |

Fixture contract check:

```sh
python3 docs/specs/tools/validate_contract_fixtures.py
python3 docs/specs/tools/validate_contract_fixtures.py docs/specs/fixtures/demo_game/manifest.json
```

Project-local bootstrap shim:

```sh
./aa validate --format json
./aa validate examples/demo_game --format sarif
./aa validate docs/specs/fixtures/empty_project --format json
./aa validate examples/demo_game --format json
./aa check examples/open_world_studio --json
./aa check examples/demo_game --json
./aa config get net.default_port --project examples/demo_game --json
./aa playtest --scenario smoke --json
./aa playtest --scenario open_world_enemy_camp --json
./aa playtest --scenario fireball_hit --json
./aa ability graph fireball --project examples/demo_game --json
./aa eval list --json
./aa eval run demo_game_add_fire_ability --json
./aa eval run open_world_studio_enemy_camp --json
```

Temporary project/config validation bridge:

```sh
python3 docs/specs/tools/aa_bootstrap.py validate --format json
python3 docs/specs/tools/aa_bootstrap.py validate examples/open_world_studio --format json
python3 docs/specs/tools/aa_bootstrap.py validate examples/demo_game --format sarif
```

Temporary context index bridge:

```sh
python3 docs/specs/tools/aa_bootstrap.py index --query "enemy camp sector" --json
python3 docs/specs/tools/aa_bootstrap.py index --query "camp guard spawn table basic melee" --scope examples/open_world_studio/assets --json
python3 docs/specs/tools/aa_bootstrap.py index --query "fire ability fireball demo_game" --json
```

Temporary Cargo check bridge:

```sh
python3 docs/specs/tools/aa_bootstrap.py check <cargo-project-root> --json
```

Temporary config discovery bridge:

```sh
python3 docs/specs/tools/aa_bootstrap.py config get net.default_port --project examples/demo_game --json
```

Temporary open-world fixture bridges:

```sh
python3 docs/specs/tools/aa_bootstrap.py world inspect --world open_world_studio --json
python3 docs/specs/tools/aa_bootstrap.py world cook --world open_world_studio --verify --json
python3 docs/specs/tools/aa_bootstrap.py world generate --template starter_open_world --output examples/generated_world_preview --name test_world --dry-run --json
python3 docs/specs/tools/aa_bootstrap.py scene list --scene examples/open_world_studio/assets/sectors/sector_0_0.ron --json
python3 docs/specs/tools/aa_bootstrap.py scene inspect sector_0_0/entity_0 --scene examples/open_world_studio/assets/sectors/sector_0_0.ron --json
python3 docs/specs/tools/aa_bootstrap.py scene patch --scene examples/open_world_studio/assets/sectors/sector_0_0.ron --patch docs/specs/fixtures/open_world_studio/add_campfire.scene_patch.json --dry-run --json
python3 docs/specs/tools/aa_bootstrap.py playtest --scenario smoke --json
python3 docs/specs/tools/aa_bootstrap.py playtest --scenario open_world_enemy_camp --json
python3 docs/specs/tools/aa_bootstrap.py playtest --scenario fireball_hit --json
python3 docs/specs/tools/aa_bootstrap.py profile summarize artifacts/profiles/open_world_enemy_camp.trace --json
python3 docs/specs/tools/aa_bootstrap.py ability graph basic_melee --project examples/open_world_studio --json
python3 docs/specs/tools/aa_bootstrap.py ability graph fireball --project examples/demo_game --json
```

Temporary eval report bridge:

```sh
python3 docs/specs/tools/aa_bootstrap.py eval list --json
python3 docs/specs/tools/aa_bootstrap.py eval run open_world_studio_enemy_camp --json
python3 docs/specs/tools/aa_bootstrap.py eval run demo_game_add_fire_ability --json
```

Bootstrap bridge tests:

```sh
python3 docs/specs/tools/test_bootstrap_cli.py
```

This bridge exists to prove the P0 validation contract before `crates/aa_cli`
is approved and implemented, including JSON and SARIF diagnostic output. The
index bridge proves the Cursor-like context gathering contract before `aa index`
exists. The config bridge proves project settings can be read with source
attribution before `aa config get` exists. The check bridge proves the structured
compiler diagnostic contract before `aa check` exists. The eval
bridge proves eval discovery and controlled prompt-to-feature report shape before
`aa eval list` / `aa eval run` exist. The open-world fixture bridges prove output contracts for inspect, cook,
world generate dry-run, scene list/inspect, scene patch dry-run, playtest, profile summary, and ability graph output, not runtime
streaming/playtest behavior. These are not substitutes for the project-local
`aa validate`, `aa index`, `aa config get`, `aa check`, `aa world`, `aa playtest`, `aa profile`,
`aa ability graph`, `aa eval list`, and `aa eval run` commands required by the gates. On macOS,
bare `aa` currently resolves to Apple Archive; use `./aa` for bootstrap evidence
until the Rust CLI owns the executable name.

## Spec Package Structure

Every `aa_*/SPEC.md` contains:

1. **Scope** — in/out of bounds
2. **UE5 reference** — responsibility source (local paths)
3. **Requirements** — numbered `REQ-{CRATE}-{NNN}` with MUST/SHALL
4. **API contract** — normative Rust traits/types
5. **Invariants** — must always hold
6. **Performance** — links to budgets
7. **Test matrix** — scenario / input / expected / automation
8. **Acceptance** — gate to mark crate "AA-ready"

## Implementation Rule

> **No subsystem is AA-ready until all its SPEC acceptance tests pass in CI.**

Spec-first TDD:

```
1. Write REQ-* from SPEC
2. Write failing integration test
3. Implement until green
4. Record in 04_ACCEPTANCE_GATES.md
```

## Bevy Version

| Field | Value |
|-------|-------|
| Target Bevy | `0.19.0-dev` (pin in `aa_engine/Cargo.toml`) |
| Rust | `1.95.0+` |
| Facade rule | Game code MUST NOT depend on Bevy internals directly except via `aa_*` public API |

## Related

- Research atlas: `docs/research/unreal_to_bevy/README.md`
- Agent rules: `AGENTS.md` (project root)
- CLI contract: research `17_agent_cli_contract.md` (superseded by `aa_cli/SPEC.md`)
- Post-AA ambition: `06_AAA_OPEN_WORLD_STUDIO.md`
- Execution order: `07_ENGINE_STUDIO_EXECUTION_PLAN.md`
- Current gate evidence: `GATE_STATUS.md`
