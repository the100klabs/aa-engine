#!/usr/bin/env python3
"""Generate an open_world_studio sector grid with 8 data layers.

Default: 32x32 sectors @ 256m = 64 km² (AS-06 / OWB scale).
Use --half 8 for the legacy 16x16 / 16 km² OWA grid.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OWS = REPO / "examples/open_world_studio"
SECTORS = OWS / "assets/sectors"
WORLD = OWS / "assets/worlds/open_world_studio.ron"

SECTOR_SIZE = 256.0
HEIGHT = 256.0
DEFAULT_HALF = 16  # coords -16..15 => 32 sectors per axis => 1024 sectors
LAYERS = [
    "terrain",
    "gameplay",
    "encounters",
    "foliage",
    "audio",
    "vfx",
    "nav",
    "lighting",
]
REQUIRED_LAYERS = ["terrain", "gameplay", "encounters"]


def ron_str(value: str) -> str:
    return json.dumps(value)


def sector_content(x: int, y: int) -> str:
    min_x = x * SECTOR_SIZE
    min_z = y * SECTOR_SIZE
    max_x = min_x + SECTOR_SIZE
    max_z = min_z + SECTOR_SIZE
    sector_id = f"sector_{x}_{y}"
    entities = ""
    if x == 0 and y == 0:
        entities = """
    entities: [
        (
            prefab: "assets/prefabs/camp_fire.ron",
            transform: (
                translation: (32.0, 0.0, -18.0),
                rotation_y_degrees: 0.0,
                scale: (1.0, 1.0, 1.0),
            ),
        ),
        (
            prefab: "assets/spawn_tables/enemy_camp_sector_0_0.ron",
            transform: (
                translation: (28.0, 0.0, -20.0),
                rotation_y_degrees: 45.0,
                scale: (1.0, 1.0, 1.0),
            ),
        ),
    ],"""
    else:
        entities = "\n    entities: [],"
    layer_list = ", ".join(ron_str(layer) for layer in LAYERS)
    return f"""SectorDescriptor(
    schema_version: 1,
    id: {ron_str(sector_id)},
    coord: ({x}, {y}),
    bounds: (min: ({min_x:.1f}, 0.0, {min_z:.1f}), max: ({max_x:.1f}, {HEIGHT:.1f}, {max_z:.1f})),
    data_layers: [{layer_list}],{entities}
)
"""


def world_content(sectors: list[tuple[int, int]], half: int) -> str:
    min_bound = -half * SECTOR_SIZE
    max_bound = half * SECTOR_SIZE
    axis = half * 2
    area_km2 = int((axis * SECTOR_SIZE / 1000.0) ** 2)
    layer_entries = []
    for layer in LAYERS:
        default = "active" if layer in {"terrain", "gameplay", "encounters", "nav"} else "loaded"
        layer_entries.append(
            f"""        (
            id: {ron_str(layer)},
            default_state: {default},
            server_authoritative: true,
        )"""
        )
    sector_refs = []
    for x, y in sectors:
        sector_id = f"sector_{x}_{y}"
        priority = 255 if x == 0 and y == 0 else (200 if abs(x) <= 1 and abs(y) <= 1 else 128)
        req = ", ".join(ron_str(layer) for layer in REQUIRED_LAYERS)
        sector_refs.append(
            f"""        (
            id: {ron_str(sector_id)},
            coord: ({x}, {y}),
            path: {ron_str(f"sectors/{sector_id}.ron")},
            required_layers: [{req}],
            priority: {priority},
        )"""
        )
    return f"""WorldDescriptor(
    schema_version: 1,
    id: "open_world_studio",
    display_name: "Open World Studio",
    description: "{area_km2} km2 streamed open-world prototype for AAA studio track.",
    bounds_m: (min: ({min_bound:.1f}, 0.0, {min_bound:.1f}), max: ({max_bound:.1f}, {HEIGHT:.1f}, {max_bound:.1f})),
    sector_size_m: {SECTOR_SIZE:.1f},
    active_window: (
        x: 3,
        y: 3,
    ),
    streaming: (
        max_activations_per_frame: 2,
        max_deactivations_per_frame: 2,
        load_latency_budget_ms: 400.0,
        crossing_hitch_budget_ms: 6.0,
        multi_source: true,
    ),
    data_layers: [
{",".join(layer_entries)}
    ],
    regions: [
        (
            id: "starter_valley",
            coord: (0, 0),
            bounds_m: (min: ({min_bound:.1f}, 0.0, {min_bound:.1f}), max: ({max_bound:.1f}, {HEIGHT:.1f}, {max_bound:.1f})),
            sectors: [
{",".join(sector_refs)}
            ],
        ),
    ],
    budgets: (
        authored_objects: 4096,
        visible_instanced_props: 16384,
        full_ai_agents: 64,
        low_lod_agents: 256,
        memory_mb: 512.0,
    ),
)
"""


def generate(half: int) -> int:
    SECTORS.mkdir(parents=True, exist_ok=True)
    coords: list[tuple[int, int]] = []
    for x in range(-half, half):
        for y in range(-half, half):
            coords.append((x, y))
            path = SECTORS / f"sector_{x}_{y}.ron"
            path.write_text(sector_content(x, y), encoding="utf-8")
    for path in SECTORS.glob("sector_*.ron"):
        stem = path.stem
        parts = stem.split("_")
        if len(parts) != 3:
            continue
        try:
            sx, sy = int(parts[1]), int(parts[2])
        except ValueError:
            continue
        if not (-half <= sx < half and -half <= sy < half):
            path.unlink()
    WORLD.write_text(world_content(coords, half), encoding="utf-8")
    print(f"generated {len(coords)} sectors ({half * 2}x{half * 2}) and {WORLD.relative_to(REPO)}")
    return len(coords)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--half",
        type=int,
        default=DEFAULT_HALF,
        help="half-axis sector count (default 16 => 32x32 / 64 km²)",
    )
    args = parser.parse_args()
    if args.half < 1 or args.half > 32:
        raise SystemExit("--half must be between 1 and 32")
    generate(args.half)


if __name__ == "__main__":
    main()
