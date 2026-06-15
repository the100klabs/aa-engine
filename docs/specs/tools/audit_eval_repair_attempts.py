#!/usr/bin/env python3
"""Audit AS-02: eval repair-attempt budgets average ≤ 2 across the eval corpus."""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
FIXTURES_DIR = REPO_ROOT / "docs/specs/fixtures"
MAX_AVERAGE = 2.0
MAX_PER_TASK = 2


def load_eval_paths() -> list[Path]:
    return sorted(FIXTURES_DIR.rglob("*.eval.json"))


def audit() -> dict[str, object]:
    errors: list[str] = []
    budgets: list[int] = []
    suites: list[dict[str, object]] = []

    for path in load_eval_paths():
        suite = json.loads(path.read_text(encoding="utf-8"))
        suite_id = str(suite.get("id", path.stem))
        suite_max = int(suite.get("max_repair_attempts", 0))
        task_budgets = [int(task.get("max_repair_attempts", 0)) for task in suite.get("tasks", [])]
        budgets.extend(task_budgets)

        if suite_max > MAX_PER_TASK:
            errors.append(f"{suite_id}: suite max_repair_attempts {suite_max} > {MAX_PER_TASK}")
        for task in suite.get("tasks", []):
            task_max = int(task.get("max_repair_attempts", 0))
            if task_max > MAX_PER_TASK:
                errors.append(
                    f"{suite_id}/{task.get('id', '<unknown>')}: task max_repair_attempts {task_max} > {MAX_PER_TASK}"
                )
            if task_max != suite_max and suite_max:
                errors.append(
                    f"{suite_id}/{task.get('id', '<unknown>')}: task budget {task_max} != suite budget {suite_max}"
                )

        suites.append(
            {
                "id": suite_id,
                "path": str(path.relative_to(REPO_ROOT)),
                "max_repair_attempts": suite_max,
                "task_budgets": task_budgets,
            }
        )

    average = sum(budgets) / len(budgets) if budgets else 0.0
    if average > MAX_AVERAGE:
        errors.append(f"repair budget average {average:.2f} exceeds {MAX_AVERAGE}")

    return {
        "ok": len(errors) == 0,
        "task_count": len(budgets),
        "budget_average": average,
        "maximum_allowed_average": MAX_AVERAGE,
        "maximum_allowed_per_task": MAX_PER_TASK,
        "suites": suites,
        "errors": errors,
    }


def main() -> int:
    result = audit()
    print(json.dumps(result, indent=2))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
