# aa_render — Subsystem Specification

> **Normative** | Priority P4

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-RENDER-010 | 4 scalability presets MUST apply atomically |
| REQ-RENDER-011 | Post stack MUST include tonemap + bloom minimum |
| REQ-RENDER-020 | Frame MUST ≤ 16.67ms @ AA Target High |
| REQ-RENDER-030 | GI MUST use probes OR baked lightmaps at AA |
| REQ-RENDER-040 | Virtual geometry MAY be used for static env (AA) |

## Acceptance: P4 gate PASS.
