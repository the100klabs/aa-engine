#!/usr/bin/env python3
"""Audit REQ-* traceability: ≥50 requirements mapped to automated test evidence."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
SPECS_DIR = REPO_ROOT / "docs/specs"
MAP_PATH = SPECS_DIR / "REQ_TEST_MAP.json"
BOOTSTRAP_TESTS = REPO_ROOT / "docs/specs/tools/test_bootstrap_cli.py"
MIN_MAPPED = 50

REQ_PATTERN = re.compile(r"REQ-[A-Z0-9]+-\d{3}")


def load_declared_req_ids() -> set[str]:
    declared: set[str] = set()
    for path in SPECS_DIR.rglob("*.md"):
        text = path.read_text(encoding="utf-8")
        declared.update(REQ_PATTERN.findall(text))
    return declared


def load_map() -> list[dict[str, str]]:
    data = json.loads(MAP_PATH.read_text(encoding="utf-8"))
    mappings = data.get("mappings", [])
    if not isinstance(mappings, list):
        raise ValueError("REQ_TEST_MAP.json mappings must be a list")
    return mappings


def verify_evidence(evidence: str) -> str | None:
    if evidence.startswith("rust_test:"):
        _, rel_path, fn_name = evidence.split(":", 2)
        path = REPO_ROOT / rel_path
        if not path.is_file():
            return f"missing rust test file: {rel_path}"
        text = path.read_text(encoding="utf-8")
        if f"fn {fn_name}" not in text:
            return f"rust test function not found: {fn_name} in {rel_path}"
        return None

    if evidence.startswith("bootstrap:"):
        test_name = evidence.split(":", 1)[1]
        text = BOOTSTRAP_TESTS.read_text(encoding="utf-8")
        if f"def {test_name}" not in text:
            return f"bootstrap test not found: {test_name}"
        return None

    if evidence.startswith("playtest:"):
        scenario = evidence.split(":", 1)[1]
        hits = 0
        for path in REPO_ROOT.rglob("playtest.rs"):
            if scenario in path.read_text(encoding="utf-8"):
                hits += 1
        ron_hits = list((REPO_ROOT / "examples").rglob(f"assets/playtests/{scenario}.ron"))
        if hits == 0 and not ron_hits:
            return f"playtest scenario not found: {scenario}"
        return None

    if evidence.startswith("cli:"):
        # Documented CLI gate command — presence in GATE_STATUS or CI is sufficient.
        command = evidence.split(":", 1)[1]
        ci = (REPO_ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        gate = (SPECS_DIR / "GATE_STATUS.md").read_text(encoding="utf-8")
        needle = command.replace("cargo run -p aa_cli -- ", "")
        if needle not in ci and needle not in gate and command not in gate:
            # Allow partial matches for quoted CLI args.
            tokens = needle.split()
            if not any(tokens[0] in text for text in (ci, gate)):
                return f"cli evidence not referenced in CI/GATE_STATUS: {command}"
        return None

    return f"unknown evidence prefix: {evidence}"


def audit() -> dict[str, object]:
    declared = load_declared_req_ids()
    mappings = load_map()
    errors: list[str] = []
    mapped_ids: list[str] = []

    for entry in mappings:
        req_id = entry.get("req_id", "")
        evidence = entry.get("evidence", "")
        if not req_id or not evidence:
            errors.append(f"invalid mapping entry: {entry}")
            continue
        if req_id not in declared:
            errors.append(f"REQ id not declared in specs: {req_id}")
        if err := verify_evidence(evidence):
            errors.append(f"{req_id}: {err}")
        mapped_ids.append(req_id)

    unique_mapped = sorted(set(mapped_ids))
    return {
        "ok": len(errors) == 0 and len(unique_mapped) >= MIN_MAPPED,
        "mapped_count": len(unique_mapped),
        "minimum_required": MIN_MAPPED,
        "errors": errors,
        "mapped_req_ids": unique_mapped,
    }


def main() -> int:
    result = audit()
    print(json.dumps(result, indent=2))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
