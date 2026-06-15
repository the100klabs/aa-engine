#!/usr/bin/env python3
"""Audit AS-06: open_world_studio world scale is 64 km² (1024 sectors @ 256m)."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
MIN_SECTORS = 1024
MIN_AREA_KM2 = 64


def audit() -> dict[str, object]:
    errors: list[str] = []
    output = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "aa_cli",
            "--",
            "world",
            "inspect",
            "--project",
            "examples/open_world_studio",
            "--world",
            "open_world_studio",
            "--json",
        ],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    payload = json.loads(output.stdout)
    sector_count = int(payload.get("sector_count", 0))
    bounds = payload.get("bounds_m", {})
    min_b = bounds.get("min", [0, 0, 0])
    max_b = bounds.get("max", [0, 0, 0])
    span_x = abs(float(max_b[0]) - float(min_b[0]))
    span_z = abs(float(max_b[2]) - float(min_b[2]))
    area_km2 = (span_x / 1000.0) * (span_z / 1000.0)

    if not payload.get("ok"):
        errors.append("world inspect returned ok=false")
    if sector_count < MIN_SECTORS:
        errors.append(f"sector_count {sector_count} < {MIN_SECTORS}")
    if area_km2 < MIN_AREA_KM2:
        errors.append(f"authored area {area_km2:.1f} km² < {MIN_AREA_KM2}")

    return {
        "ok": len(errors) == 0,
        "sector_count": sector_count,
        "area_km2": round(area_km2, 2),
        "minimum_sectors": MIN_SECTORS,
        "minimum_area_km2": MIN_AREA_KM2,
        "errors": errors,
    }


def main() -> int:
    result = audit()
    print(json.dumps(result, indent=2))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
