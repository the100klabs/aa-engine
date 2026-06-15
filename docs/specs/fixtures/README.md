# Spec Fixtures

These files are machine-readable examples for the schemas in `docs/specs/schemas/`.
They are not production assets and should not be loaded by the runtime directly.

Use fixtures to build CLI contract tests before real `assets/*.ron` content exists.
When the project-local `aa` CLI is implemented, these examples should become
schema-validation and golden-output tests.

## Open World Studio Fixtures

`open_world_studio/` models the first AAA/Open World studio slice:

```text
Add an enemy camp to sector 0/0 in open_world_studio.
```

The fixture set covers:

- agent eval input
- playtest scenario input
- world inspect output
- world cook output
- profile summary output
- playtest result output
- manifest mapping fixtures to formal schemas

The real implementation must still create RON assets under `assets/`, run the
project-local CLI, and record passing gate evidence in `docs/specs/GATE_STATUS.md`.

## Local Contract Check

Until the project-local `aa validate` command exists, run:

```sh
python3 docs/specs/tools/validate_contract_fixtures.py
```

This stdlib-only check validates the fixture manifest, applies the schema subset
used by these contracts, and rejects absolute or parent-directory paths.

## Platform Boot Sign-off (P0-06)

`platform_boot_signoff.json` documents automated Linux CI boot proxies (headless
world inspect, playtest smoke, platform config merge) and the manual Win/macOS
visual boot checklist. Audit with:

```sh
python3 docs/specs/tools/audit_platform_boot.py
```
