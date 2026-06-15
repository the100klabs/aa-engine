#!/usr/bin/env python3
"""Audit P0-06: platform boot sign-off fixture, CI proxy wiring, and manual checklist."""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
SIGNOFF_PATH = REPO_ROOT / "docs/specs/fixtures/platform_boot_signoff.json"
CI_PATH = REPO_ROOT / ".github/workflows/ci.yml"


def rust_test_fn_exists(command: str) -> str | None:
    prefix = "cargo test -p aa_core --test "
    if not command.startswith(prefix):
        return None
    rest = command[len(prefix) :]
    parts = rest.split(maxsplit=1)
    if len(parts) != 2:
        return f"cannot parse rust test command: {command}"
    test_module, fn_name = parts
    test_path = REPO_ROOT / "crates/aa_core/tests" / f"{test_module}.rs"
    if not test_path.is_file():
        return f"missing rust integration test file: {test_path.relative_to(REPO_ROOT)}"
    text = test_path.read_text(encoding="utf-8")
    if f"fn {fn_name}" not in text:
        return f"rust test function not found: {fn_name} in {test_path.relative_to(REPO_ROOT)}"
    return None


def verify_automated_entry(entry: dict[str, object], ci_text: str) -> str | None:
    entry_id = str(entry.get("id", "<unknown>"))
    status = str(entry.get("status", ""))
    command = str(entry.get("command", ""))

    if status != "pass":
        return f"{entry_id}: automated evidence status is not pass"

    if command.startswith("cargo test -p aa_core --test "):
        if err := rust_test_fn_exists(command):
            return f"{entry_id}: {err}"
        test_module = command[len("cargo test -p aa_core --test ") :].split(maxsplit=1)[0]
        if test_module not in ci_text:
            return f"{entry_id}: rust test module not referenced in ci.yml ({test_module})"
        return None

    if command.startswith("cargo run -p aa_cli -- "):
        cli_needle = command.replace("cargo run -p aa_cli -- ", "")
        if cli_needle not in ci_text:
            return f"{entry_id}: cli command not referenced in ci.yml"
        return None

    return f"{entry_id}: unsupported command form"


def audit() -> dict[str, object]:
    errors: list[str] = []
    signoff = json.loads(SIGNOFF_PATH.read_text(encoding="utf-8"))
    ci_text = CI_PATH.read_text(encoding="utf-8")

    if signoff.get("gate") != "P0-06":
        errors.append("signoff gate must be P0-06")

    automated = signoff.get("automated_evidence", [])
    if len(automated) < 3:
        errors.append("expected at least 3 automated_evidence entries")

    passing = 0
    for entry in automated:
        if err := verify_automated_entry(entry, ci_text):
            errors.append(err)
        elif entry.get("status") == "pass":
            passing += 1

    for rel in signoff.get("platform_configs", {}).get("paths", []):
        path = REPO_ROOT / rel
        if not path.is_file():
            errors.append(f"missing platform config: {rel}")

    manual = signoff.get("manual_checklist", [])
    pending_manual = [item for item in manual if item.get("status") == "pending"]
    passed_manual = [item for item in manual if item.get("status") == "pass"]

    gate_status = signoff.get("gate_status", "partial")
    if gate_status == "pass" and pending_manual:
        errors.append("gate_status is pass but manual checklist items are still pending")

    return {
        "ok": len(errors) == 0,
        "gate_id": signoff.get("gate"),
        "gate_status": gate_status,
        "automated_passing": passing,
        "automated_total": len(automated),
        "manual_pending": [item.get("id", item.get("platform")) for item in pending_manual],
        "manual_passed": [item.get("id", item.get("platform")) for item in passed_manual],
        "gate_status_recommendation": "PARTIAL" if pending_manual else "PASS",
        "errors": errors,
    }


def main() -> int:
    result = audit()
    print(json.dumps(result, indent=2))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
