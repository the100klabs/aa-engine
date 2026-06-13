# aa_assets — Subsystem Specification

> **Normative** | Priority P0

## UE5 Reference: `AssetRegistry`, DDC, cook pipeline

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-ASSET-001 | `AssetRegistry` MUST index all assets under `assets/` at startup |
| REQ-ASSET-002 | `asset_manifest.json` MUST list id, kind, path, hash, deps |
| REQ-ASSET-003 | RON loaders MUST reject unknown `schema_version` |
| REQ-ASSET-004 | Migrator MUST exist per schema version bump |
| REQ-ASSET-005 | glTF loader MUST register via Bevy `AssetServer` |
| REQ-ASSET-006 | Hot reload MUST trigger `AssetEvent::Modified` consumers |
| REQ-ASSET-007 | `resolve(path)` MUST return `None` for missing refs (validator uses this) |
| REQ-ASSET-008 | Import command MUST update manifest hash |
| REQ-ASSET-009 | Root `aa.project.toml` MUST validate against `schemas/project.schema.json` |
| REQ-ASSET-010 | Project manifest paths MUST be relative project paths, not absolute filesystem paths |
| REQ-ASSET-011 | Project manifest soft refs MUST resolve when optional `default_experience` or `startup_scene` are present |
| REQ-ASSET-012 | `assets/asset_manifest.json` MUST validate against `schemas/asset_manifest.schema.json` when present |
| REQ-ASSET-013 | Asset manifest IDs and paths MUST be relative project paths, not absolute filesystem paths |
| REQ-ASSET-014 | Asset manifest dependency refs MUST resolve to another manifest `id` or validator MUST report `REF_MISSING` |
| REQ-ASSET-015 | Asset manifest cook artifacts MUST include kind, path, and hash for generated runtime artifacts |

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-ASSET-01 | Load ability RON | deserializes | unit |
| T-ASSET-02 | Bad schema version | error | unit |
| T-ASSET-03 | Manifest deps | matches RON refs | integration |
| T-ASSET-04 | Load project manifest | validates schema + roots | integration |
| T-ASSET-05 | Bad project path | absolute or parent path rejected | unit |
| T-ASSET-06 | Load asset manifest | validates schema + dependency refs | integration |
| T-ASSET-07 | Bad asset manifest path | absolute or parent path rejected | unit |
| T-ASSET-08 | Missing dependency | unresolved dep returns `REF_MISSING` | integration |

## Acceptance: P0 when T-ASSET-01–08 green.
