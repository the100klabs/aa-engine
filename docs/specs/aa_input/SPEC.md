# aa_input — Subsystem Specification

> **Normative** | Priority P1 | UE ref: `EnhancedInput` plugin

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-INPUT-001 | Input actions MUST be semantic (`Fire`, not `MouseLeft`) |
| REQ-INPUT-002 | Mapping contexts MUST stack by priority (higher wins) |
| REQ-INPUT-003 | `ActionEvent` MUST emit in `PreUpdate` |
| REQ-INPUT-004 | Ability input buffer MUST consume `ActionEvent` in `FixedPrePhysics` |
| REQ-INPUT-005 | UI context MUST block lower-priority gameplay contexts when active |
| REQ-INPUT-006 | Input context assets MUST validate against `schemas/input_context.schema.json` |
| REQ-INPUT-007 | Input action names MUST be semantic actions, not physical device names |
| REQ-INPUT-008 | Mapping contexts referenced by experiences, action sets, or PawnData MUST resolve before play |

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-INPUT-01 | Context priority | UI blocks move | integration |
| T-INPUT-02 | Gamepad + KB | same action fires | integration |
| T-INPUT-03 | Input schema | malformed context rejected | unit |
| T-INPUT-04 | Semantic action audit | physical action name rejected | unit |

## Acceptance: P1 when T-INPUT-01–04 green.
