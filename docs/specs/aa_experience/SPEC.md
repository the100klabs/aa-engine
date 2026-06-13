# aa_experience — Subsystem Specification

> **Normative** | Priority P1 | UE ref: `LyraExperienceDefinition.h`, `LyraExperienceManagerComponent.h`

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-EXP-001 | Experience MUST load from RON per `schemas/experience.schema.json` |
| REQ-EXP-002 | Load FSM MUST match: Unloaded → Loading → LoadingFeatures → ExecutingActions → Loaded |
| REQ-EXP-003 | `ExperienceReady` event MUST fire once per load |
| REQ-EXP-004 | Game feature crates MUST register via `aa.project.toml` enabled list |
| REQ-EXP-005 | Actions MUST execute in order: action_sets then inline actions |
| REQ-EXP-006 | Load MUST complete ≤ 5s for reference experience |
| REQ-EXP-007 | Action set assets MUST validate against `schemas/action_set.schema.json` |
| REQ-EXP-008 | Experience `action_sets` refs MUST resolve before executing inline actions |

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-EXP-01 | Load shooter_dm | ExperienceReady | integration |
| T-EXP-02 | Grant abilities action | abilities in registry | integration |
| T-EXP-03 | Action set schema | malformed action set rejected | unit |
| T-EXP-04 | Missing action set ref | validation reports `REF_MISSING` | integration |

## Acceptance: P1 when T-EXP-01–04 green.
