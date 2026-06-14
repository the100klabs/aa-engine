#!/usr/bin/env python3
"""P3-08 audit: count schema-valid playtest scenarios (target ≥20)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from aa_bootstrap import load_ron_subset
from schema_subset import load_json, validate_schema

REPO_ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = REPO_ROOT / "docs/specs/schemas/playtest_scenario.schema.json"
MIN_SCENARIOS = 20


def discover_playtest_files() -> list[Path]:
    roots = [
        REPO_ROOT / "examples/demo_game/assets/playtests",
        REPO_ROOT / "examples/open_world_studio/assets/playtests",
        REPO_ROOT / "examples/demo_game_contract/assets/playtests",
    ]
    files: list[Path] = []
    for root in roots:
        if root.is_dir():
            files.extend(sorted(root.glob("*.ron")))
    return files


def audit() -> dict[str, object]:
    schema = load_json(SCHEMA_PATH)
    errors: list[str] = []
    scenarios: list[dict[str, str]] = []

    for path in discover_playtest_files():
        rel = path.relative_to(REPO_ROOT).as_posix()
        try:
            data = load_ron_subset(path)
            validate_schema(schema, data, schema, "$")
            scenario_id = str(data.get("id", path.stem))
            scenarios.append({"id": scenario_id, "path": rel})
        except Exception as exc:  # noqa: BLE001 - aggregate audit failures
            errors.append(f"{rel}: {exc}")

    unique_ids = sorted({entry["id"] for entry in scenarios})
    return {
        "ok": len(errors) == 0 and len(unique_ids) >= MIN_SCENARIOS,
        "scenario_count": len(unique_ids),
        "minimum_required": MIN_SCENARIOS,
        "scenarios": scenarios,
        "errors": errors,
    }


def main() -> int:
    result = audit()
    print(json.dumps(result, indent=2))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
