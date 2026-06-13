# aa_animation — Subsystem Specification

> **Normative** | Priority P1

## Requirements (P1 / AA)

| ID | Requirement |
|----|-------------|
| REQ-ANIM-001 | Locomotion FSM MUST support Idle/Walk/Run/Jump |
| REQ-ANIM-002 | Speed param MUST drive playback rate |
| REQ-ANIM-003 | `AnimNotify` events MUST fire at clip timestamps |
| REQ-ANIM-004 | Montage MUST override FSM while playing (AA) |
| REQ-ANIM-005 | 16 skeletons MUST eval ≤ 2ms (AA) |
| REQ-ANIM-006 | Motion matching MUST search pose DB in ≤ 1ms (AA P4) |

## Acceptance: P1 when locomotion playtest green; AA when REQ-ANIM-005 green.
