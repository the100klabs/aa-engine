# ⚠️ RESEARCH ONLY — NOT NORMATIVE

> **These documents are UE5 analysis and design research.**  
> **Do NOT implement from these files directly.**

## Use the AA Specs instead

**Normative build contract:** [`docs/specs/README.md`](../../specs/README.md)

| Research (informative) | Spec (normative) |
|------------------------|------------------|
| Conceptual sketches | `REQ-*` with MUST/SHALL |
| Example RON in prose | JSON Schema in `docs/specs/schemas/` |
| MVP/AA narrative | Measurable acceptance gates |
| "Conceptual Rust" | API contracts + test matrices |

## How to use this folder

1. **Understand why** — read research for UE5 → Bevy rationale
2. **Build what** — implement from `docs/specs/aa_*/SPEC.md`
3. **Verify** — pass gates in `docs/specs/04_ACCEPTANCE_GATES.md`

## Document tiers

| Tier | Status |
|------|--------|
| `00`–`11` | Research appendix |
| `12`–`17` | Implementation guides (superseded by specs where conflict) |
| `AGENTS.md` | Copy to project root; updated to reference specs |

---

Continue to [README.md](./README.md) for navigation.
