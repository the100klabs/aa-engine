# AA Engine

Open-source Bevy ECS gameplay framework targeting **AA indie quality** with a path to **AAA open-world + Cursor-like agent studio**. Normative specs live in [`docs/specs/`](docs/specs/README.md); research rationale in [`docs/research/unreal_to_bevy/`](docs/research/unreal_to_bevy/).

## Quick start

```bash
# Bootstrap CLI (full contract: validate, index, eval, world inspect, …)
./aa validate examples/demo_game --format json
python3 docs/specs/tools/test_bootstrap_cli.py

# Rust workspace
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo run -p aa_cli -- validate examples/demo_game
cargo run -p aa_cli -- playtest --project examples/demo_game --scenario smoke --duration 12

# Playable combat vertical slice (windowed)
cargo run -p demo_game

# Headless playtest harness
AA_PLAYTEST=1 AA_PLAYTEST_DURATION=12 cargo run -p demo_game
```

## Workspace layout

```
aa_engine/
├── AGENTS.md                 # Agent rules (read before editing)
├── aa                        # Bootstrap CLI shim → docs/specs/tools/aa_bootstrap.py
├── aa.project.toml           # Workspace manifest
├── config/                   # Engine base config layers
├── crates/
│   ├── aa_core … aa_net      # Runtime plugins
│   └── aa_cli                # Rust CLI (check, validate, playtest)
├── docs/
│   ├── specs/                # Normative AA/AAA specs + JSON schemas + bootstrap
│   └── research/unreal_to_bevy/
└── examples/
    ├── demo_game/            # Playable Phase 1 combat slice (Rust runtime)
    └── open_world_studio/    # Open-world contract package (AAA track)
```

## Tiers (see `docs/specs/00_AA_DEFINITION.md`)

| Tier | Goal |
|------|------|
| **AA** | Shippable indie systems (GAS, net, 4–16 km² stream, agent validate/playtest) |
| **AAA** | Post-AA open world (16–64 km²) + agent studio eval loop (`06_AAA_OPEN_WORLD_STUDIO.md`) |

## CLI

| Entry | Role |
|-------|------|
| `./aa …` | Bootstrap contract CLI (schemas, SARIF, evals, world inspect) |
| `cargo run -p aa_cli -- …` | Rust CLI (compile check, project validate, runtime playtest) |

## Documentation hierarchy

1. **`docs/specs/`** — normative (implement + test against)
2. **`docs/research/unreal_to_bevy/`** — informative UE5 analysis
3. **`AGENTS.md`** — mandatory agent workflow

Gate evidence: [`docs/specs/GATE_STATUS.md`](docs/specs/GATE_STATUS.md)

## License

MIT OR Apache-2.0
