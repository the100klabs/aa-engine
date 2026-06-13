# 17 — Agent & CLI Contract

> **Machine interface for your Cursor-like AA game app.** Agents never guess — they call these commands and read structured output.

## Design Principles

1. **JSON stdout** for agent parsing; human text to stderr
2. **Exit codes** are contractual
3. **Idempotent** commands safe to retry
4. **Path allowlist** — agents cannot touch `target/`, `.git/`
5. **Dry-run** on destructive ops

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation failed |
| 2 | Compile failed |
| 3 | Playtest failed |
| 4 | Invalid arguments |
| 5 | Internal error |

---

## Command Reference

### `aa new <name>`

Create project from template.

```bash
aa new my_game --template shooter
```

**Output (JSON):**
```json
{
  "ok": true,
  "project_path": "/path/my_game",
  "files_created": 42
}
```

---

### `aa check [path]`

Run `cargo check` + type validation.

```bash
aa check examples/demo_game
```

**Output:**
```json
{
  "ok": true,
  "errors": [],
  "warnings": [],
  "duration_ms": 3200
}
```

On failure:
```json
{
  "ok": false,
  "errors": [
    {
      "file": "crates/aa_ability/src/lib.rs",
      "line": 45,
      "message": "expected `AbilityId`, found `String`"
    }
  ]
}
```

---

### `aa validate [path]`

Asset + schema validation without compile.

```bash
aa validate examples/demo_game --format sarif
```

**Output (SARIF excerpt):**
```json
{
  "runs": [{
    "results": [{
      "ruleId": "REF_MISSING",
      "message": { "text": "abilities/fireball refs missing effect 'effects/burning'" },
      "locations": [{ "physicalLocation": { "artifactLocation": { "uri": "assets/abilities/fireball.ron" }}}]
    }]
  }]
}
```

---

### `aa index [path]`

Build or refresh project index.

```bash
aa index . --query "fire abilities"
```

**Output:**
```json
{
  "ok": true,
  "query": "fire abilities",
  "hits": [
    {
      "kind": "asset",
      "id": "abilities/fireball",
      "path": "assets/abilities/fireball.ron",
      "score": 0.95
    },
    {
      "kind": "rust_symbol",
      "name": "register_fireball",
      "path": "crates/aa_ability/src/abilities/fireball.rs",
      "score": 0.82
    }
  ]
}
```

---

### `aa scene list [path]`

List entities in open scene or scene file.

```bash
aa scene list --scene assets/scenes/arena_01.ron --filter "SpawnPoint"
```

**Output:**
```json
{
  "entities": [
    {
      "id": "entity_0",
      "name": "SpawnPoint_01",
      "components": ["Transform", "SpawnPoint"]
    }
  ]
}
```

---

### `aa scene inspect <entity_id>`

```bash
aa scene inspect entity_0 --scene assets/scenes/arena_01.ron
```

**Output:**
```json
{
  "entity_id": "entity_0",
  "components": {
    "Transform": { "translation": [0.0, 0.0, 0.0] },
    "SpawnPoint": { "team": 0, "tag": "Spawn.Player" }
  }
}
```

---

### `aa scene patch`

Apply validated patch to scene.

```bash
aa scene patch --scene assets/scenes/arena_01.ron --patch patch.json --dry-run
```

**patch.json:**
```json
{
  "ops": [
    {
      "op": "add_component",
      "entity": "entity_0",
      "component": "SpawnPoint",
      "value": { "team": 1, "tag": "Spawn.Player" }
    }
  ]
}
```

**Output:**
```json
{
  "ok": true,
  "undo_token": "undo_8f3a2b",
  "applied_ops": 1
}
```

---

### `aa playtest`

```bash
aa playtest --scenario intro_combat --duration 30s --headless
```

**Output:**
```json
{
  "ok": true,
  "scenario": "intro_combat",
  "duration_secs": 30,
  "assertions": [
    { "name": "player_alive", "passed": true },
    { "name": "enemy_defeated", "passed": true }
  ],
  "artifacts": {
    "log": "/tmp/playtest_abc.log",
    "trace": "/tmp/playtest_abc.tracy"
  }
}
```

---

### `aa run`

```bash
aa run --project examples/demo_game --role client
aa run --project examples/demo_game --role dedicated_server
aa run --project examples/demo_game --role editor
```

---

### `aa config get <key>`

```bash
aa config get net.default_port --project examples/demo_game
```

**Output:**
```json
{ "key": "net.default_port", "value": 7777, "source": "config/engine.toml" }
```

---

### `aa tags list`

```bash
aa tags list --project examples/demo_game
```

---

### `aa ability graph`

Dependency graph for ability/effect refs.

```bash
aa ability graph abilities/fireball
```

**Output:**
```json
{
  "root": "abilities/fireball",
  "nodes": [
    { "id": "abilities/fireball", "kind": "ability" },
    { "id": "effects/burning", "kind": "effect" },
    { "id": "GameplayCue.Fire.Impact", "kind": "tag" }
  ],
  "edges": [
    { "from": "abilities/fireball", "to": "effects/burning", "label": "applies" }
  ]
}
```

---

## JSON-RPC Server (Editor Mode)

**Endpoint:** `ws://127.0.0.1:9742` (default)  
**Crate:** `aa_editor_protocol`

### Methods

| Method | Params | Returns |
|--------|--------|---------|
| `scene.list` | `{ filter?: string }` | `{ entities: [...] }` |
| `scene.inspect` | `{ entity_id: string }` | `{ components: {...} }` |
| `scene.patch` | `{ ops: [...], dry_run?: bool }` | `{ ok, undo_token }` |
| `project.validate` | `{}` | `{ ok, errors: [...] }` |
| `project.index` | `{ query: string }` | `{ hits: [...] }` |
| `session.play` | `{}` | `{ ok }` |
| `session.stop` | `{}` | `{ ok }` |
| `playtest.run` | `{ scenario, duration_secs }` | Playtest result |

### Example request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "scene.list",
  "params": { "filter": "Pawn" }
}
```

---

## Agent Repair Loop (canonical)

```
1. aa index --query "<task context>"
2. aa check
3. [agent edits files]
4. aa check
5. aa validate
6. aa playtest --scenario <relevant>
7. if fail → read artifacts.log → goto 3 (max 5 iterations)
8. present diff to human
```

---

## Cursor Integration (`AGENTS.md` rules)

Place in project root. Key rules for agents:

| Rule | Command to run |
|------|----------------|
| After gameplay asset edit | `aa validate` |
| After Rust edit | `aa check` |
| Before claiming done | `aa playtest --scenario smoke` |
| Before ability work | `aa ability graph <id>` |
| Scene changes | `aa scene patch --dry-run` first |

---

## MCP Tool Mapping (future)

| MCP tool name | CLI equivalent |
|---------------|----------------|
| `aa_validate` | `aa validate` |
| `aa_check` | `aa check` |
| `aa_playtest` | `aa playtest` |
| `aa_search` | `aa index --query` |
| `aa_scene_inspect` | `aa scene inspect` |
| `aa_scene_patch` | `aa scene patch` |

---

## Security Allowlist

Agents may read/write:

```
aa.project.toml
config/**
assets/**
src/**
crates/**
examples/**
```

Agents may NOT write:

```
target/**
.git/**
Cargo.lock        # without explicit instruction
config/user.toml  # user-local
```

---

## Versioning

```bash
aa --version
```

```json
{ "aa_cli": "0.1.0", "schema_version": 1, "bevy": "0.16.0" }
```

Breaking CLI changes bump `aa_cli` minor version.

---

*Implement commands incrementally in `crates/aa_cli`. Start with `validate` and `check` in Phase 0.*
