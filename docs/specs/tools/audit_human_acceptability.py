#!/usr/bin/env python3
"""Audit AS-04: human acceptability review sample ≥80% accepted without manual rewrite."""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
SAMPLE_PATH = REPO_ROOT / "docs/specs/fixtures/agent_review_sample.json"
MIN_ACCEPT_RATE = 0.8


def audit() -> dict[str, object]:
    errors: list[str] = []
    sample = json.loads(SAMPLE_PATH.read_text(encoding="utf-8"))
    reviews = sample.get("reviews", [])
    if not isinstance(reviews, list) or not reviews:
        errors.append("review sample must include at least one review entry")

    accepted = 0
    checked: list[dict[str, object]] = []
    for entry in reviews:
        review_id = str(entry.get("id", "<unknown>"))
        artifact = str(entry.get("artifact", ""))
        if not artifact:
            errors.append(f"{review_id}: missing artifact path")
            continue
        if not (REPO_ROOT / artifact).is_file():
            errors.append(f"{review_id}: artifact missing on disk: {artifact}")
        if "accepted" not in entry:
            errors.append(f"{review_id}: missing accepted boolean")
            continue
        is_accepted = bool(entry["accepted"])
        if is_accepted:
            accepted += 1
        checked.append({"id": review_id, "artifact": artifact, "accepted": is_accepted})

    total = len(checked)
    rate = accepted / total if total else 0.0
    minimum = float(sample.get("minimum_accept_rate", MIN_ACCEPT_RATE))
    if rate < minimum:
        errors.append(f"accept rate {rate:.2%} below minimum {minimum:.0%}")

    return {
        "ok": len(errors) == 0,
        "review_count": total,
        "accepted_count": accepted,
        "accept_rate": rate,
        "minimum_accept_rate": minimum,
        "reviews": checked,
        "errors": errors,
    }


def main() -> int:
    result = audit()
    print(json.dumps(result, indent=2))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
