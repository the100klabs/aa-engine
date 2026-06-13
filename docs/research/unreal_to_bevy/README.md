# Unreal-to-Bevy Architecture Atlas

Clean-room research for building a **UE5-class AA engine** on Bevy + a **Cursor-like game studio app**.

## Quick Start (I want to build NOW)

1. **[12_integration_blueprint.md](./12_integration_blueprint.md)** — how everything connects
2. **[15_phase0_bootstrap_guide.md](./15_phase0_bootstrap_guide.md)** — create `aa_engine` workspace
3. **[AGENTS.md](./AGENTS.md)** — copy to project root for AI assistants
4. **[16_anti_patterns_and_decisions.md](./16_anti_patterns_and_decisions.md)** — avoid fatal mistakes

## Document Map

### Concepts (what UE5 does, what Bevy needs)

| Doc | Topic |
|-----|-------|
| [00_overview](./00_overview.md) | Atlas index + crate topology |
| [01_engine_architecture](./01_engine_architecture.md) | Modules, plugins, build, config, assets |
| [02_object_model_vs_ecs](./02_object_model_vs_ecs.md) | UObject/Actor vs ECS |
| [03_gameplay_framework](./03_gameplay_framework.md) | GAS, tags, input, Lyra |
| [04_world_streaming](./04_world_streaming.md) | World Partition, Mass |
| [05_rendering_stack](./05_rendering_stack.md) | Nanite/Lumen concepts, Bevy render |
| [06_animation_stack](./06_animation_stack.md) | AnimBP, motion matching |
| [07_physics_simulation](./07_physics_simulation.md) | Chaos, character movement |
| [08_networking](./08_networking.md) | Replication, Iris |
| [09_editor_tooling](./09_editor_tooling.md) | UnrealEd domains |
| [10_ai_native_game_studio](./10_ai_native_game_studio.md) | Agent-driven studio |
| [11_bevy_roadmap](./11_bevy_roadmap.md) | 33-month phased plan |

### Implementation (how to build it)

| Doc | Topic |
|-----|-------|
| [12_integration_blueprint](./12_integration_blueprint.md) | Boot sequence, entity graph, data flows |
| [13_data_schemas](./13_data_schemas.md) | RON/TOML formats |
| [14_system_schedule_spec](./14_system_schedule_spec.md) | Bevy schedule law |
| [15_phase0_bootstrap_guide](./15_phase0_bootstrap_guide.md) | Workspace scaffold |
| [16_anti_patterns_and_decisions](./16_anti_patterns_and_decisions.md) | ADRs + DO NOT list |
| [17_agent_cli_contract](./17_agent_cli_contract.md) | CLI/JSON-RPC for agents |

## Product Vision

```
┌─────────────────────────────────────────────┐
│  Cursor-like Game Studio (your app)         │
│  ┌─────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ Agent   │ │ Viewport │ │ Asset browser│  │
│  │ chat    │ │ (Bevy)   │ │ + inspector  │  │
│  └────┬────┘ └────┬─────┘ └──────┬───────┘  │
│       └───────────┼──────────────┘          │
│                   ▼                         │
│         aa_cli + aa_agent (validate/play)   │
└─────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────┐
│  aa_engine workspace (Rust crates)          │
│  core → scene → gameplay → ability → net    │
└─────────────────────────────────────────────┘
                    ▼
              Bevy 0.16+ / wgpu
```

## Legal

Clean-room research only. UE5 concepts extracted from local tree at `Engine/Source/`, `Engine/Plugins/`, `Samples/Games/Lyra/`. No UE source code in deliverables.
