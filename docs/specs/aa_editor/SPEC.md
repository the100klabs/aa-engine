# aa_editor — Subsystem Specification

> **Normative** | Priority P3

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-EDIT-001 | Editor MUST share runtime plugins with `SessionMode::Editing` gate |
| REQ-EDIT-002 | Viewport MUST render 3D scene |
| REQ-EDIT-003 | Hierarchy MUST list entities |
| REQ-EDIT-004 | Inspector MUST edit `Transform` minimum |
| REQ-EDIT-005 | Save MUST write lossless scene RON |
| REQ-EDIT-006 | Play/Stop MUST toggle `SessionMode` |
| REQ-EDIT-007 | JSON-RPC MUST expose `scene.*` methods per `aa_cli/SPEC.md` |

## Acceptance: P3 gate PASS.
