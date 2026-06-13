# aa_physics — Subsystem Specification

> **Normative** | Priority P1 | Backend: `bevy_rapier3d`

## UE5 Reference: `CharacterMovementComponent.h`, `PhysScene_Chaos.h`

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-PHY-001 | Physics MUST step at 60 Hz in `FixedUpdate` |
| REQ-PHY-002 | `CharacterMovement` MUST support Walk, Falling modes minimum |
| REQ-PHY-003 | Ground detection MUST use shape cast downward |
| REQ-PHY-004 | Jump MUST apply `jump_velocity` on ground only |
| REQ-PHY-005 | Slope limit MUST be configurable per `PawnData` |
| REQ-PHY-006 | Collision layers MUST use 32-bit groups matrix |
| REQ-PHY-007 | Projectiles MUST use sensor colliders + hit events |

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-PHY-01 | Walk + jump | no fall through floor 60s | playtest |
| T-PHY-02 | Projectile hit | `CollisionEvent` fired | integration |

## Acceptance: P1 when T-PHY-01–02 green.
