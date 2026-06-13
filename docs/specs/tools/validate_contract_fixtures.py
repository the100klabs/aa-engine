#!/usr/bin/env python3
"""Validate AA spec contract fixtures with no third-party dependencies.

This is intentionally smaller than a full JSON Schema implementation. It checks
the schema keywords used by the AA fixture contracts so early CLI work has a
repeatable target before the project-local `aa validate` command exists.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from schema_subset import ValidationError, load_json, project_relative, validate_schema, walk_project_paths


REPO_ROOT = Path(__file__).resolve().parents[3]


def validate_manifest(manifest_path: Path) -> list[tuple[Path, Path]]:
    manifest = load_json(manifest_path)
    walk_project_paths(manifest, str(manifest_path))

    if manifest.get("schema_version") != 1:
        raise ValidationError(f"{manifest_path}: schema_version must be 1")
    if not isinstance(manifest.get("fixtures"), list) or not manifest["fixtures"]:
        raise ValidationError(f"{manifest_path}: fixtures must be a non-empty array")

    pairs: list[tuple[Path, Path]] = []
    seen_ids: set[str] = set()
    for index, item in enumerate(manifest["fixtures"]):
        if not isinstance(item, dict):
            raise ValidationError(f"{manifest_path}.fixtures[{index}]: expected object")
        for key in ("id", "fixture", "schema", "kind"):
            if key not in item:
                raise ValidationError(f"{manifest_path}.fixtures[{index}]: missing {key!r}")
        if item["id"] in seen_ids:
            raise ValidationError(f"{manifest_path}.fixtures[{index}]: duplicate id {item['id']!r}")
        seen_ids.add(item["id"])

        fixture_path = REPO_ROOT / item["fixture"]
        schema_path = REPO_ROOT / item["schema"]
        if not fixture_path.is_file():
            raise ValidationError(f"{manifest_path}: missing fixture {item['fixture']}")
        if not schema_path.is_file():
            raise ValidationError(f"{manifest_path}: missing schema {item['schema']}")
        pairs.append((fixture_path, schema_path))
    return pairs


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate AA spec contract fixtures")
    parser.add_argument(
        "manifest",
        nargs="?",
        default="docs/specs/fixtures/open_world_studio/manifest.json",
        help="Project-relative fixture manifest path",
    )
    args = parser.parse_args()

    manifest_path = REPO_ROOT / args.manifest
    try:
        pairs = validate_manifest(manifest_path)
        for fixture_path, schema_path in pairs:
            fixture = load_json(fixture_path)
            schema = load_json(schema_path)
            validate_schema(schema, fixture, schema, "$")
            walk_project_paths(fixture, str(fixture_path.relative_to(REPO_ROOT)))
            print(f"ok {fixture_path.relative_to(REPO_ROOT)}")
    except ValidationError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    print("contract_fixtures_ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
