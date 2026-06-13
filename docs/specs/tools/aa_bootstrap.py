#!/usr/bin/env python3
"""Bootstrap AA CLI behavior before the Rust `aa_cli` crate exists."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import time
import tomllib
from pathlib import Path
from typing import Any

from schema_subset import ValidationError, load_json, validate_schema


REPO_ROOT = Path(__file__).resolve().parents[3]
DEFAULT_OPEN_WORLD_EVAL = REPO_ROOT / "docs/specs/fixtures/open_world_studio/add_enemy_camp.eval.json"
OPEN_WORLD_FIXTURE_ROOT = REPO_ROOT / "docs/specs/fixtures/open_world_studio"
DEFAULT_DEMO_GAME_EVAL = REPO_ROOT / "docs/specs/fixtures/demo_game/add_fire_ability.eval.json"
DEMO_GAME_FIXTURE_ROOT = REPO_ROOT / "docs/specs/fixtures/demo_game"


def rel(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def rel_to(path: Path, base: Path) -> str:
    try:
        return str(path.resolve().relative_to(base.resolve()))
    except ValueError:
        return rel(path)


def diagnostic(
    *,
    code: str,
    severity: str,
    message: str,
    path: Path,
    project_root: Path,
    schema: Path | None = None,
    help_text: str | None = None,
) -> dict[str, Any]:
    item: dict[str, Any] = {
        "code": code,
        "severity": severity,
        "message": message,
        "path": rel_to(path, project_root),
    }
    if schema is not None:
        item["schema"] = rel(schema)
    if help_text is not None:
        item["help"] = help_text
    return item


def load_toml(path: Path) -> dict[str, Any]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        raise ValidationError(str(exc)) from exc


class RonSubsetParser:
    """Parse the small RON subset used by AA contract fixtures."""

    def __init__(self, text: str) -> None:
        self.text = text
        self.index = 0

    def parse(self) -> Any:
        value = self.parse_value()
        self.skip_ws()
        if self.index != len(self.text):
            raise ValidationError(f"Unexpected token at byte {self.index}")
        return value

    def skip_ws(self) -> None:
        while self.index < len(self.text):
            if self.text[self.index].isspace():
                self.index += 1
                continue
            if self.text.startswith("//", self.index):
                next_line = self.text.find("\n", self.index)
                self.index = len(self.text) if next_line == -1 else next_line + 1
                continue
            break

    def parse_value(self) -> Any:
        self.skip_ws()
        if self.index >= len(self.text):
            raise ValidationError("Unexpected end of RON")
        char = self.text[self.index]
        if char == "(":
            return self.parse_object()
        if char == "[":
            return self.parse_array()
        if char == '"':
            return self.parse_string()
        if char == "-" or char.isdigit():
            return self.parse_number()
        ident = self.parse_identifier()
        if ident == "true":
            return True
        if ident == "false":
            return False
        if ident == "null":
            return None
        raise ValidationError(f"Unsupported bare identifier '{ident}' at byte {self.index}")

    def parse_object(self) -> dict[str, Any]:
        self.expect("(")
        result: dict[str, Any] = {}
        while True:
            self.skip_ws()
            if self.peek() == ")":
                self.index += 1
                return result
            key = self.parse_key()
            self.skip_ws()
            self.expect(":")
            result[key] = self.parse_value()
            self.skip_ws()
            if self.peek() == ",":
                self.index += 1

    def parse_array(self) -> list[Any]:
        self.expect("[")
        result: list[Any] = []
        while True:
            self.skip_ws()
            if self.peek() == "]":
                self.index += 1
                return result
            result.append(self.parse_value())
            self.skip_ws()
            if self.peek() == ",":
                self.index += 1

    def parse_key(self) -> str:
        self.skip_ws()
        if self.peek() == '"':
            return self.parse_string()
        return self.parse_identifier()

    def parse_identifier(self) -> str:
        self.skip_ws()
        start = self.index
        while self.index < len(self.text):
            char = self.text[self.index]
            if char.isalnum() or char == "_":
                self.index += 1
            else:
                break
        if self.index == start:
            raise ValidationError(f"Expected identifier at byte {self.index}")
        return self.text[start:self.index]

    def parse_string(self) -> str:
        self.expect('"')
        chars: list[str] = []
        while self.index < len(self.text):
            char = self.text[self.index]
            self.index += 1
            if char == '"':
                return "".join(chars)
            if char == "\\":
                if self.index >= len(self.text):
                    raise ValidationError("Unterminated string escape")
                escaped = self.text[self.index]
                self.index += 1
                escape_map = {
                    '"': '"',
                    "\\": "\\",
                    "n": "\n",
                    "r": "\r",
                    "t": "\t",
                }
                if escaped not in escape_map:
                    raise ValidationError(f"Unsupported string escape '\\{escaped}'")
                chars.append(escape_map[escaped])
            else:
                chars.append(char)
        raise ValidationError("Unterminated string")

    def parse_number(self) -> int | float:
        self.skip_ws()
        start = self.index
        if self.peek() == "-":
            self.index += 1
        while self.index < len(self.text) and self.text[self.index].isdigit():
            self.index += 1
        if self.peek() == ".":
            self.index += 1
            while self.index < len(self.text) and self.text[self.index].isdigit():
                self.index += 1
        if self.peek() in {"e", "E"}:
            self.index += 1
            if self.peek() in {"+", "-"}:
                self.index += 1
            while self.index < len(self.text) and self.text[self.index].isdigit():
                self.index += 1
        token = self.text[start:self.index]
        if not token or token in {"-", ".", "-."}:
            raise ValidationError(f"Expected number at byte {start}")
        return float(token) if any(marker in token for marker in ".eE") else int(token)

    def expect(self, expected: str) -> None:
        self.skip_ws()
        if self.peek() != expected:
            raise ValidationError(f"Expected '{expected}' at byte {self.index}")
        self.index += 1

    def peek(self) -> str:
        if self.index >= len(self.text):
            return ""
        return self.text[self.index]


def load_ron_subset(path: Path) -> Any:
    return RonSubsetParser(path.read_text(encoding="utf-8")).parse()


def validate_toml_file(
    path: Path,
    schema_path: Path,
    diagnostics: list[dict[str, Any]],
    project_root: Path,
) -> dict[str, Any] | None:
    if not path.is_file():
        diagnostics.append(
            diagnostic(
                code="FILE_MISSING",
                severity="error",
                message="Required TOML file is missing",
                path=path,
                project_root=project_root,
                schema=schema_path,
            )
        )
        return None

    try:
        data = load_toml(path)
        schema = load_json(schema_path)
        validate_schema(schema, data, schema, "$")
        return data
    except ValidationError as exc:
        diagnostics.append(
            diagnostic(
                code="SCHEMA_INVALID",
                severity="error",
                message=str(exc),
                path=path,
                project_root=project_root,
                schema=schema_path,
            )
        )
        return None


def validate_ron_file(
    path: Path,
    schema_path: Path,
    diagnostics: list[dict[str, Any]],
    project_root: Path,
) -> Any | None:
    try:
        data = load_ron_subset(path)
        schema = load_json(schema_path)
        validate_schema(schema, data, schema, "$")
        return data
    except ValidationError as exc:
        diagnostics.append(
            diagnostic(
                code="SCHEMA_INVALID",
                severity="error",
                message=str(exc),
                path=path,
                project_root=project_root,
                schema=schema_path,
            )
        )
        return None


def validate_soft_ref(
    *,
    source_path: Path,
    ref: str | None,
    diagnostics: list[dict[str, Any]],
    project_root: Path,
    field: str,
) -> None:
    if not ref:
        return
    ref_path = project_root / ref
    if ref_path.exists():
        return
    diagnostics.append(
        diagnostic(
            code="REF_MISSING",
            severity="error",
            message=f"Missing soft ref in {field}: {ref}",
            path=source_path,
            project_root=project_root,
            help_text="Create the referenced asset or update the soft ref.",
        )
    )


def validate_asset_refs(
    asset_path: Path,
    data: dict[str, Any],
    diagnostics: list[dict[str, Any]],
    project_root: Path,
    asset_kind: str,
) -> None:
    if asset_kind == "world":
        for region in data.get("regions", []):
            for sector in region.get("sectors", []):
                validate_soft_ref(
                    source_path=asset_path,
                    ref=sector.get("path"),
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field="regions[].sectors[].path",
                )
    elif asset_kind == "sector":
        for entity in data.get("entities", []):
            validate_soft_ref(
                source_path=asset_path,
                ref=entity.get("prefab"),
                diagnostics=diagnostics,
                project_root=project_root,
                field="entities[].prefab",
            )
    elif asset_kind == "spawn_table":
        for entry in data.get("entries", []):
            for field in ("pawn", "ai_profile", "prefab"):
                validate_soft_ref(
                    source_path=asset_path,
                    ref=entry.get(field),
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field=f"entries[].{field}",
                )
    elif asset_kind == "ai_profile":
        behavior = data.get("behavior", {})
        validate_soft_ref(
            source_path=asset_path,
            ref=behavior.get("patrol_route"),
            diagnostics=diagnostics,
            project_root=project_root,
            field="behavior.patrol_route",
        )
        combat = data.get("combat", {})
        for ability in combat.get("abilities", []):
            validate_soft_ref(
                source_path=asset_path,
                ref=ability,
                diagnostics=diagnostics,
                project_root=project_root,
                field="combat.abilities[]",
            )
        for effect in combat.get("effects_on_spawn", []):
            validate_soft_ref(
                source_path=asset_path,
                ref=effect,
                diagnostics=diagnostics,
                project_root=project_root,
                field="combat.effects_on_spawn[]",
            )
    elif asset_kind == "playtest":
        setup = data.get("setup", {})
        for ref in setup.values():
            validate_soft_ref(
                source_path=asset_path,
                ref=ref,
                diagnostics=diagnostics,
                project_root=project_root,
                field="setup",
            )
        for timed_action in data.get("input_script", []):
            action = timed_action.get("action", {})
            spawn = action.get("SpawnPlayer")
            if spawn:
                validate_soft_ref(
                    source_path=asset_path,
                    ref=spawn.get("pawn"),
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field="input_script[].SpawnPlayer.pawn",
                )
            ability = action.get("ActivateAbility")
            if ability:
                validate_soft_ref(
                    source_path=asset_path,
                    ref=ability.get("ability"),
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field="input_script[].ActivateAbility.ability",
                )
    elif asset_kind == "ability":
        cost_effect = data.get("cost_effect")
        if isinstance(cost_effect, str):
            validate_soft_ref(
                source_path=asset_path,
                ref=cost_effect,
                diagnostics=diagnostics,
                project_root=project_root,
                field="cost_effect",
            )
    elif asset_kind == "pawn":
        for field in ("attribute_sets", "input_contexts"):
            for ref in data.get(field, []):
                validate_soft_ref(
                    source_path=asset_path,
                    ref=ref,
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field=f"{field}[]",
                )
    elif asset_kind == "action_set":
        for action in data.get("actions", []):
            grant = action.get("GrantAbilitySet") if isinstance(action, dict) else None
            if isinstance(grant, dict):
                for ref in grant.get("abilities", []):
                    validate_soft_ref(
                        source_path=asset_path,
                        ref=ref,
                        diagnostics=diagnostics,
                        project_root=project_root,
                        field="actions[].GrantAbilitySet.abilities[]",
                    )
            input_context = action.get("AddInputContext") if isinstance(action, dict) else None
            if isinstance(input_context, dict):
                validate_soft_ref(
                    source_path=asset_path,
                    ref=input_context.get("context"),
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field="actions[].AddInputContext.context",
                )
            ui_layout = action.get("RegisterUiLayout") if isinstance(action, dict) else None
            if isinstance(ui_layout, dict):
                validate_soft_ref(
                    source_path=asset_path,
                    ref=ui_layout.get("layout"),
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field="actions[].RegisterUiLayout.layout",
                )
            load_scene = action.get("LoadScene") if isinstance(action, dict) else None
            if isinstance(load_scene, dict):
                validate_soft_ref(
                    source_path=asset_path,
                    ref=load_scene.get("scene"),
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field="actions[].LoadScene.scene",
                )
    elif asset_kind == "experience":
        validate_soft_ref(
            source_path=asset_path,
            ref=data.get("default_pawn"),
            diagnostics=diagnostics,
            project_root=project_root,
            field="default_pawn",
        )
        for ref in data.get("action_sets", []):
            validate_soft_ref(
                source_path=asset_path,
                ref=ref,
                diagnostics=diagnostics,
                project_root=project_root,
                field="action_sets[]",
            )
        for action in data.get("actions", []):
            grant = action.get("GrantAbilitySet") if isinstance(action, dict) else None
            if isinstance(grant, dict):
                for ref in grant.get("abilities", []):
                    validate_soft_ref(
                        source_path=asset_path,
                        ref=ref,
                        diagnostics=diagnostics,
                        project_root=project_root,
                        field="actions[].GrantAbilitySet.abilities[]",
                    )
            input_context = action.get("AddInputContext") if isinstance(action, dict) else None
            if isinstance(input_context, dict):
                validate_soft_ref(
                    source_path=asset_path,
                    ref=input_context.get("context"),
                    diagnostics=diagnostics,
                    project_root=project_root,
                    field="actions[].AddInputContext.context",
                )


def declared_attribute_names(attribute_set: dict[str, Any]) -> set[str]:
    names: set[str] = set()
    for attribute in attribute_set.get("attributes", []):
        if isinstance(attribute, dict) and isinstance(attribute.get("name"), str):
            names.add(attribute["name"])
    return names


def effect_attribute_root(attribute: str) -> str:
    return attribute.split(".", 1)[0]


def validate_effect_modifier_attributes(
    effect_assets: list[tuple[Path, dict[str, Any]]],
    declared_attributes: set[str],
    diagnostics: list[dict[str, Any]],
    project_root: Path,
) -> None:
    if not declared_attributes:
        return

    for effect_path, effect in effect_assets:
        for modifier in effect.get("modifiers", []):
            if not isinstance(modifier, dict) or not isinstance(modifier.get("attribute"), str):
                continue
            attribute = modifier["attribute"]
            if attribute in declared_attributes or effect_attribute_root(attribute) in declared_attributes:
                continue
            diagnostics.append(
                diagnostic(
                    code="ATTR_UNKNOWN",
                    severity="error",
                    message=(
                        f"GameplayEffect modifier attribute '{attribute}' is not declared by any reachable "
                        "AttributeSet asset"
                    ),
                    path=effect_path,
                    project_root=project_root,
                    help_text="Add the attribute to an AttributeSet asset or update the GameplayEffect modifier.",
                )
            )


def validate_project(project_root: Path) -> dict[str, Any]:
    start = time.perf_counter()
    diagnostics: list[dict[str, Any]] = []
    schemas = REPO_ROOT / "docs/specs/schemas"

    project_path = project_root / "aa.project.toml"
    project_data = validate_toml_file(project_path, schemas / "project.schema.json", diagnostics, project_root)
    engine_config = project_data.get("engine", {}) if project_data else {}
    config_root_name = engine_config.get("config_root", "config")
    assets_root_name = engine_config.get("assets_root", "assets")

    config_schema_pairs = [
        ("engine.toml", "config_engine.schema.json"),
        ("game.toml", "config_game.schema.json"),
        ("input.toml", "config_input.schema.json"),
        ("scalability.toml", "config_scalability.schema.json"),
    ]
    config_root = project_root / config_root_name
    if not config_root.is_dir():
        diagnostics.append(
            diagnostic(
                code="DIR_MISSING",
                severity="error",
                message="Config root declared by aa.project.toml must exist",
                path=config_root,
                project_root=project_root,
                help_text="Create the config root or update [engine].config_root.",
            )
        )
    for config_name, schema_name in config_schema_pairs:
        validate_toml_file(config_root / config_name, schemas / schema_name, diagnostics, project_root)

    assets_root = project_root / assets_root_name
    if not assets_root.is_dir():
        diagnostics.append(
            diagnostic(
                code="DIR_MISSING",
                severity="error",
                message="Assets root declared by aa.project.toml must exist",
                path=assets_root,
                project_root=project_root,
                help_text="Create the assets root or update [engine].assets_root.",
            )
        )
    else:
        declared_attributes: set[str] = set()
        effect_assets: list[tuple[Path, dict[str, Any]]] = []
        asset_schema_pairs = [
            ("worlds/*.ron", "world.schema.json", "world"),
            ("sectors/*.ron", "sector.schema.json", "sector"),
            ("spawn_tables/*.ron", "spawn_table.schema.json", "spawn_table"),
            ("ai/*.ron", "ai_profile.schema.json", "ai_profile"),
            ("playtests/*.ron", "playtest_scenario.schema.json", "playtest"),
            ("attributes/*.ron", "attribute_set.schema.json", "attribute_set"),
            ("input/contexts/*.ron", "input_context.schema.json", "input_context"),
            ("action_sets/*.ron", "action_set.schema.json", "action_set"),
            ("experiences/*.ron", "experience.schema.json", "experience"),
            ("pawns/*.ron", "pawn_data.schema.json", "pawn"),
            ("prefabs/*.ron", "prefab.schema.json", "prefab"),
            ("abilities/*.ron", "gameplay_ability.schema.json", "ability"),
            ("effects/*.ron", "gameplay_effect.schema.json", "effect"),
            ("data/tags.ron", "tag_dictionary.schema.json", "tag_dictionary"),
        ]
        for glob_pattern, schema_name, asset_kind in asset_schema_pairs:
            for asset_path in sorted(assets_root.glob(glob_pattern)):
                asset_data = validate_ron_file(asset_path, schemas / schema_name, diagnostics, project_root)
                if isinstance(asset_data, dict):
                    if asset_kind == "attribute_set":
                        declared_attributes.update(declared_attribute_names(asset_data))
                    elif asset_kind == "effect":
                        effect_assets.append((asset_path, asset_data))
                    validate_asset_refs(asset_path, asset_data, diagnostics, project_root, asset_kind)
        validate_effect_modifier_attributes(effect_assets, declared_attributes, diagnostics, project_root)

    duration_ms = int((time.perf_counter() - start) * 1000)
    error_count = sum(1 for item in diagnostics if item["severity"] == "error")
    warning_count = sum(1 for item in diagnostics if item["severity"] == "warning")
    return {
        "ok": error_count == 0,
        "error_count": error_count,
        "warning_count": warning_count,
        "diagnostics": diagnostics,
        "duration_ms": duration_ms,
    }


def sarif_level(severity: str) -> str:
    if severity == "error":
        return "error"
    if severity == "warning":
        return "warning"
    return "note"


def validation_to_sarif(validation_result: dict[str, Any]) -> dict[str, Any]:
    diagnostics = validation_result.get("diagnostics", [])
    rule_messages: dict[str, str] = {}
    results: list[dict[str, Any]] = []

    for item in diagnostics:
        code = str(item.get("code", "AA_VALIDATION"))
        message = str(item.get("message", "Validation diagnostic"))
        rule_messages.setdefault(code, message)

        physical_location: dict[str, Any] = {
            "artifactLocation": {
                "uri": str(item.get("path", "aa.project.toml")),
            },
        }
        span = item.get("span")
        if isinstance(span, dict) and span.get("line"):
            region: dict[str, int] = {"startLine": int(span["line"])}
            if span.get("column"):
                region["startColumn"] = int(span["column"])
            physical_location["region"] = region

        properties: dict[str, Any] = {}
        for source_key in ("schema", "help"):
            if source_key in item:
                properties[source_key] = item[source_key]

        result_item: dict[str, Any] = {
            "ruleId": code,
            "level": sarif_level(str(item.get("severity", "note"))),
            "message": {"text": message},
            "locations": [
                {
                    "physicalLocation": physical_location,
                }
            ],
        }
        if properties:
            result_item["properties"] = properties
        results.append(result_item)

    rules = [
        {
            "id": code,
            "name": code,
            "shortDescription": {"text": message},
        }
        for code, message in sorted(rule_messages.items())
    ]

    return {
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "aa validate",
                        "semanticVersion": "0.1.0-bootstrap",
                        "informationUri": "https://aa-engine.dev/specs/aa_cli",
                        "rules": rules,
                    }
                },
                "results": results,
            }
        ],
    }


def query_terms(query: str) -> list[str]:
    return [term for term in re.findall(r"[a-z0-9_]+", query.lower()) if len(term) > 1]


def index_roots(project_root: Path, requested_path: str | None) -> list[Path]:
    if requested_path:
        path = Path(requested_path)
        if not path.is_absolute():
            path = project_root / path
        return [path.resolve()]
    roots = [
        REPO_ROOT / "docs/specs",
        REPO_ROOT / "docs/research/unreal_to_bevy",
        project_root / "aa.project.toml",
        project_root / "config",
        project_root / "assets",
        project_root / "src",
        REPO_ROOT / "examples/demo_game",
        REPO_ROOT / "examples/demo_game_contract",
        REPO_ROOT / "examples/open_world_studio",
    ]
    return sorted(set(path.resolve() for path in roots))


def indexable_files(roots: list[Path]) -> list[Path]:
    allowed_suffixes = {".md", ".json", ".toml", ".ron", ".rs"}
    blocked_parts = {"target", ".git"}
    files: list[Path] = []
    for root in roots:
        if not root.exists():
            continue
        if root.is_file():
            candidates = [root]
        else:
            candidates = [path for path in root.rglob("*") if path.is_file()]
        for path in candidates:
            if any(part in blocked_parts for part in path.parts):
                continue
            if path.suffix in allowed_suffixes:
                files.append(path)
    return sorted(set(files))


def classify_hit(path: Path) -> str:
    rel_path = rel(path)
    name = path.name
    if rel_path.endswith("GATE_STATUS.md") or "ACCEPTANCE_GATES" in name:
        return "gate"
    if "/schemas/" in f"/{rel_path}" or name.endswith(".schema.json"):
        return "schema"
    if name.endswith(".eval.json") or "/evals/" in f"/{rel_path}":
        return "eval"
    if name.endswith(".playtest.json") or "/playtests/" in f"/{rel_path}":
        return "playtest"
    if path.suffix == ".toml":
        return "config_key"
    if "/docs/specs/" in f"/{rel_path}" or name == "SPEC.md":
        return "spec"
    if path.suffix == ".rs":
        return "code_symbol"
    if "/assets/" in f"/{rel_path}":
        return "asset"
    return "doc"


def extract_title(lines: list[str], fallback: str) -> str:
    for line in lines[:30]:
        stripped = line.strip()
        if stripped.startswith("#"):
            return stripped.lstrip("#").strip()
    return fallback


def relation_targets(text: str) -> list[dict[str, str]]:
    relations: list[dict[str, str]] = []
    for command in sorted(set(re.findall(r"aa [a-z]+(?: [a-z]+)?", text))):
        relations.append({"kind": "references", "target": command})
    for schema in sorted(set(re.findall(r"schemas/[A-Za-z0-9_./-]+\.schema\.json", text))):
        relations.append({"kind": "validates", "target": schema})
    return relations[:8]


def summarize_line(line: str, max_len: int = 220) -> str:
    summary = " ".join(line.strip().split())
    if len(summary) > max_len:
        return summary[: max_len - 3] + "..."
    return summary


def query_index(project_root: Path, query: str, requested_path: str | None = None) -> dict[str, Any]:
    start = time.perf_counter()
    terms = query_terms(query)
    hits: list[dict[str, Any]] = []
    warnings: list[dict[str, str]] = []

    if not terms:
        warnings.append({"code": "EMPTY_QUERY", "message": "Query had no searchable terms"})

    for path in indexable_files(index_roots(project_root, requested_path)):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            warnings.append({"code": "READ_FAILED", "message": str(exc), "path": rel(path)})
            continue

        lowered_path = rel(path).lower()
        lowered_text = text.lower()
        path_score = sum(1 for term in terms if term in lowered_path)
        text_score = sum(lowered_text.count(term) for term in terms)
        if terms and path_score == 0 and text_score == 0:
            continue

        lines = text.splitlines()
        best_line_number = 1
        best_line = lines[0] if lines else rel(path)
        best_line_score = -1
        for index, line in enumerate(lines, start=1):
            line_lower = line.lower()
            line_score = sum(line_lower.count(term) for term in terms)
            if line_score > best_line_score:
                best_line_score = line_score
                best_line_number = index
                best_line = line

        normalized = (text_score + (path_score * 2)) / max(len(terms), 1)
        score = round(min(1.0, normalized / 8.0), 4)
        if score <= 0:
            continue

        hit_id = f"{rel(path)}:{best_line_number}"
        hit = {
            "id": hit_id,
            "kind": classify_hit(path),
            "path": rel(path),
            "title": extract_title(lines, path.name),
            "score": score,
            "summary": summarize_line(best_line),
            "span": {"line_start": best_line_number},
            "relations": relation_targets(best_line),
            "tags": sorted({term for term in terms if term in lowered_text or term in lowered_path}),
            "stale": False,
        }
        hits.append(hit)

    hits.sort(key=lambda item: (-item["score"], item["path"], item["span"]["line_start"]))
    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": True,
        "query": query,
        "duration_ms": duration_ms,
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z"),
        "index_version": "bootstrap-1",
        "hits": hits[:20],
        "warnings": warnings,
    }


def span_path(file_name: str, workspace: Path) -> str:
    path = Path(file_name)
    if not path.is_absolute():
        path = workspace / path
    return rel_to(path, workspace)


def cargo_span(span: dict[str, Any], workspace: Path) -> dict[str, Any] | None:
    line_start = span.get("line_start")
    column_start = span.get("column_start")
    file_name = span.get("file_name")
    if not file_name or not line_start or not column_start:
        return None

    mapped: dict[str, Any] = {
        "path": span_path(file_name, workspace),
        "line_start": line_start,
        "column_start": column_start,
    }
    for source_key, dest_key in (
        ("line_end", "line_end"),
        ("column_end", "column_end"),
        ("is_primary", "is_primary"),
        ("label", "label"),
    ):
        if source_key in span and span[source_key] is not None:
            mapped[dest_key] = span[source_key]
    return mapped


def cargo_child(child: dict[str, Any], workspace: Path) -> dict[str, Any]:
    mapped: dict[str, Any] = {
        "severity": child.get("level", "note"),
        "message": child.get("message", ""),
    }
    spans = [cargo_span(span, workspace) for span in child.get("spans", [])]
    spans = [span for span in spans if span is not None]
    if spans:
        mapped["spans"] = spans
    return mapped


def cargo_diagnostic(message: dict[str, Any], package: str | None, target: str | None, workspace: Path) -> dict[str, Any]:
    code = message.get("code")
    mapped: dict[str, Any] = {
        "code": code.get("code") if isinstance(code, dict) and code.get("code") else message.get("level", "message"),
        "severity": message.get("level", "note"),
        "message": message.get("message", ""),
    }
    if package:
        mapped["package"] = package
    if target:
        mapped["target"] = target

    spans = [cargo_span(span, workspace) for span in message.get("spans", [])]
    spans = [span for span in spans if span is not None]
    if spans:
        mapped["spans"] = spans

    children = [cargo_child(child, workspace) for child in message.get("children", [])]
    children = [child for child in children if child["message"]]
    if children:
        mapped["children"] = children

    return mapped


def check_project(project_root: Path) -> dict[str, Any]:
    start = time.perf_counter()
    command = ["cargo", "check", "--workspace", "--message-format=json"]
    diagnostics: list[dict[str, Any]] = []
    warning_count = 0
    error_count = 0
    lock_path = project_root / "Cargo.lock"
    had_lock = lock_path.exists()

    try:
        with tempfile.TemporaryDirectory(prefix="aa-bootstrap-target-") as target_dir:
            env = {**os.environ, "CARGO_TARGET_DIR": target_dir}
            completed = subprocess.run(
                command,
                cwd=project_root,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                env=env,
            )
    except FileNotFoundError:
        duration_ms = int((time.perf_counter() - start) * 1000)
        return {
            "ok": False,
            "error_count": 1,
            "warning_count": 0,
            "duration_ms": duration_ms,
            "workspace": ".",
            "command": " ".join(command),
            "diagnostics": [
                {
                    "code": "CARGO_NOT_FOUND",
                    "severity": "error",
                    "message": "cargo executable was not found on PATH",
                }
            ],
        }
    finally:
        if not had_lock and lock_path.exists():
            lock_path.unlink()

    for line in completed.stdout.splitlines():
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("reason") != "compiler-message":
            continue

        message = event.get("message", {})
        level = message.get("level", "note")
        if level == "error":
            error_count += 1
        elif level == "warning":
            warning_count += 1
        if level in {"error", "warning", "note", "help"}:
            target = event.get("target", {}).get("name") if isinstance(event.get("target"), dict) else None
            diagnostics.append(cargo_diagnostic(message, event.get("package_id"), target, project_root))

    if completed.returncode != 0 and error_count == 0:
        error_count = 1
        message = completed.stderr.strip() or "cargo check failed without structured compiler diagnostics"
        diagnostics.append(
            {
                "code": "CARGO_CHECK_FAILED",
                "severity": "error",
                "message": message,
            }
        )

    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": completed.returncode == 0,
        "error_count": error_count,
        "warning_count": warning_count,
        "duration_ms": duration_ms,
        "workspace": ".",
        "command": " ".join(command),
        "diagnostics": diagnostics,
    }


def load_schema_validated_json(fixture_path: Path, schema_name: str) -> dict[str, Any]:
    payload = load_json(fixture_path)
    schema = load_json(REPO_ROOT / "docs/specs/schemas" / schema_name)
    validate_schema(schema, payload, schema, "$")
    return payload


def project_path_safe(value: str) -> bool:
    return bool(value) and not value.startswith("/") and ".." not in Path(value).parts and "\\" not in value


def infer_project_root_from_scene(scene_path: Path) -> tuple[Path, str]:
    resolved = scene_path.resolve()
    parts = resolved.parts
    if "assets" in parts:
        assets_index = parts.index("assets")
        project_root = Path(*parts[:assets_index])
        return project_root, rel_to(resolved, project_root)
    return resolved.parent.parent if resolved.parent.name else resolved.parent, resolved.name


def scene_patch_error(message: str, code: str = "SCENE_PATCH_INVALID", path: str = "patch.json") -> dict[str, Any]:
    return {
        "code": code,
        "severity": "error",
        "message": message,
        "path": path,
    }


def safe_diagnostic_path(path: Path, project_root: Path, fallback: str = "patch.json") -> str:
    candidate = rel_to(path, project_root)
    return candidate if project_path_safe(candidate) else fallback


def scene_patch_op_report(index: int, op: dict[str, Any]) -> tuple[dict[str, Any], list[str]]:
    [(kind, value)] = op.items()
    entity_id = value.get("entity_id", "<unknown>") if isinstance(value, dict) else "<unknown>"
    affected_files: list[str] = []
    if kind == "InstantiatePrefab" and isinstance(value, dict):
        prefab = value.get("prefab")
        if isinstance(prefab, str):
            affected_files.append(prefab)
    return {
        "index": index,
        "kind": kind,
        "entity_id": str(entity_id),
        "affected_files": affected_files,
    }, affected_files


def scene_patch_dry_run(scene_arg: str, patch_arg: str, dry_run: bool) -> dict[str, Any]:
    start = time.perf_counter()
    scene_path = Path(scene_arg)
    if not scene_path.is_absolute():
        scene_path = (REPO_ROOT / scene_path).resolve()
    patch_path = Path(patch_arg)
    if not patch_path.is_absolute():
        patch_path = (REPO_ROOT / patch_path).resolve()

    project_root, scene_project_path = infer_project_root_from_scene(scene_path)
    patch_diag_path = safe_diagnostic_path(patch_path, project_root)
    diagnostics: list[dict[str, Any]] = []
    patch_id = "unknown"
    target_path = scene_project_path if project_path_safe(scene_project_path) else "scene"
    affected_files: list[str] = []
    affected_entities: list[str] = []
    op_reports: list[dict[str, Any]] = []

    if not dry_run:
        diagnostics.append(scene_patch_error("Bootstrap scene patch only supports --dry-run.", code="DRY_RUN_REQUIRED"))

    if not scene_path.is_file():
        diagnostics.append(scene_patch_error(f"Scene target does not exist: {scene_project_path}", code="FILE_MISSING", path=scene_project_path))

    try:
        patch = load_json(patch_path)
        schema = load_json(REPO_ROOT / "docs/specs/schemas/scene_patch.schema.json")
        validate_schema(schema, patch, schema, "$")
        patch_id = str(patch.get("id", patch_id))
        target_path = patch.get("target", {}).get("path", target_path)
        if not project_path_safe(target_path):
            diagnostics.append(scene_patch_error(f"Patch target path is outside the project allowlist: {target_path}", path=patch_diag_path))
        if target_path != scene_project_path:
            diagnostics.append(
                scene_patch_error(
                    f"Patch target {target_path} does not match --scene {scene_project_path}",
                    code="TARGET_MISMATCH",
                    path=patch_diag_path,
                )
            )

        affected_files = [target_path]
        for index, op in enumerate(patch.get("ops", [])):
            report, op_files = scene_patch_op_report(index, op)
            op_reports.append(report)
            affected_entities.append(report["entity_id"])
            affected_files.extend(op_files)
            for path in op_files:
                if not project_path_safe(path):
                    diagnostics.append(scene_patch_error(f"Patch op path is outside the project allowlist: {path}", path=patch_diag_path))
                elif not (project_root / path).exists():
                    diagnostics.append(scene_patch_error(f"Patch op referenced file does not exist: {path}", code="REF_MISSING", path=patch_diag_path))
    except ValidationError as exc:
        diagnostics.append(scene_patch_error(str(exc), path=patch_diag_path))
    except OSError as exc:
        diagnostics.append(scene_patch_error(str(exc), code="FILE_MISSING", path=patch_diag_path))

    affected_files = sorted(set(affected_files))
    affected_entities = sorted(set(affected_entities))
    token_input = "|".join([patch_id, target_path, *affected_files, *affected_entities])
    undo_token = "undo:dry-run:" + hashlib.sha256(token_input.encode("utf-8")).hexdigest()[:16]
    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": not any(item["severity"] == "error" for item in diagnostics),
        "dry_run": dry_run,
        "patch_id": patch_id,
        "target": target_path,
        "affected_files": affected_files,
        "affected_entities": affected_entities,
        "ops": op_reports,
        "undo_token": undo_token,
        "diagnostics": diagnostics,
        "duration_ms": duration_ms,
    }


def scene_kind_and_id(data: Any, scene_path: Path) -> tuple[str, str]:
    if isinstance(data, dict) and "coord" in data and "entities" in data:
        return "sector", str(data.get("id", scene_path.stem))
    if isinstance(data, dict) and "children" in data:
        return "prefab", str(data.get("id", scene_path.stem))
    return "scene", scene_path.stem


def component_types(components: Any) -> list[str]:
    if isinstance(components, dict):
        return sorted(str(key) for key in components)
    return []


def scene_entities_from_data(data: Any, scene_path: Path) -> tuple[str, str, list[dict[str, Any]]]:
    kind, root_id = scene_kind_and_id(data, scene_path)
    entities: list[dict[str, Any]] = []

    if kind == "sector" and isinstance(data, dict):
        layers = [str(layer) for layer in data.get("data_layers", [])]
        for index, item in enumerate(data.get("entities", [])):
            if not isinstance(item, dict):
                continue
            entity_id = f"{root_id}/entity_{index}"
            prefab = item.get("prefab")
            entity: dict[str, Any] = {
                "id": entity_id,
                "name": Path(prefab).stem if isinstance(prefab, str) else entity_id,
                "layers": layers,
            }
            if isinstance(prefab, str):
                entity["prefab"] = prefab
            if isinstance(item.get("transform"), dict):
                entity["transform"] = item["transform"]
            entities.append(entity)
        return kind, root_id, entities

    if kind == "prefab" and isinstance(data, dict):
        for index, child in enumerate(data.get("children", [])):
            if not isinstance(child, dict):
                continue
            name = str(child.get("name", f"entity_{index}"))
            entity_id = f"{root_id}/{name}"
            entity = {
                "id": entity_id,
                "name": name,
                "component_types": component_types(child.get("components")),
                "components": child.get("components", {}),
                "children": [
                    f"{entity_id}/{grandchild.get('name', f'entity_{child_index}')}"
                    for child_index, grandchild in enumerate(child.get("children", []))
                    if isinstance(grandchild, dict)
                ],
            }
            entities.append(entity)
        return kind, root_id, entities

    return kind, root_id, entities


def scene_load_for_query(scene_arg: str) -> tuple[Path, str, str, list[dict[str, Any]], list[dict[str, Any]], float]:
    start = time.perf_counter()
    scene_path = Path(scene_arg)
    if not scene_path.is_absolute():
        scene_path = (REPO_ROOT / scene_path).resolve()
    project_root, scene_project_path = infer_project_root_from_scene(scene_path)
    diagnostics: list[dict[str, Any]] = []
    try:
        data = load_ron_subset(scene_path)
        kind, _root_id, entities = scene_entities_from_data(data, scene_path)
    except (OSError, ValidationError) as exc:
        kind = "scene"
        entities = []
        diagnostics.append(
            {
                "code": "SCENE_READ_FAILED",
                "severity": "error",
                "message": str(exc),
                "path": scene_project_path if project_path_safe(scene_project_path) else "scene",
            }
        )
    return project_root, scene_project_path, kind, entities, diagnostics, start


def scene_entity_summary(entity: dict[str, Any]) -> dict[str, Any]:
    summary: dict[str, Any] = {"id": entity["id"]}
    for key in ("name", "prefab", "layers", "component_types"):
        if key in entity:
            summary[key] = entity[key]
    return summary


def scene_list(scene_arg: str, filter_text: str | None = None) -> dict[str, Any]:
    project_root, scene_project_path, kind, entities, diagnostics, start = scene_load_for_query(scene_arg)
    del project_root
    if filter_text:
        lowered = filter_text.lower()
        entities = [
            entity
            for entity in entities
            if lowered in entity.get("id", "").lower()
            or lowered in entity.get("name", "").lower()
            or lowered in entity.get("prefab", "").lower()
            or any(lowered in component.lower() for component in entity.get("component_types", []))
        ]
    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": not any(item["severity"] == "error" for item in diagnostics),
        "scene": scene_project_path,
        "kind": kind,
        "entity_count": len(entities),
        "entities": [scene_entity_summary(entity) for entity in entities],
        "diagnostics": diagnostics,
        "duration_ms": duration_ms,
    }


def scene_inspect(scene_arg: str, entity_id: str) -> dict[str, Any]:
    project_root, scene_project_path, kind, entities, diagnostics, start = scene_load_for_query(scene_arg)
    del project_root
    found = next((entity for entity in entities if entity.get("id") == entity_id), None)
    if found is None:
        diagnostics.append(
            {
                "code": "ENTITY_NOT_FOUND",
                "severity": "error",
                "message": f"Scene entity id was not found: {entity_id}",
                "path": scene_project_path,
            }
        )
    duration_ms = int((time.perf_counter() - start) * 1000)
    result: dict[str, Any] = {
        "ok": found is not None and not any(item["severity"] == "error" for item in diagnostics),
        "scene": scene_project_path,
        "kind": kind,
        "entity_id": entity_id,
        "diagnostics": diagnostics,
        "duration_ms": duration_ms,
    }
    if found is not None:
        result["entity"] = found
    return result


def known_open_world(value: str) -> bool:
    return value in {
        "open_world_studio",
        "assets/worlds/open_world_studio.ron",
        "examples/open_world_studio/assets/worlds/open_world_studio.ron",
    }


def world_inspect_fixture(world: str) -> dict[str, Any]:
    if not known_open_world(world):
        raise ValidationError(f"unknown bootstrap world fixture {world!r}")
    return load_schema_validated_json(OPEN_WORLD_FIXTURE_ROOT / "world_inspect.result.json", "world_inspect_result.schema.json")


def world_cook_fixture(world: str) -> dict[str, Any]:
    if not known_open_world(world):
        raise ValidationError(f"unknown bootstrap world fixture {world!r}")
    return load_schema_validated_json(OPEN_WORLD_FIXTURE_ROOT / "world_cook.result.json", "world_cook_result.schema.json")


def slug_id(value: str) -> str:
    slug = re.sub(r"[^A-Za-z0-9_./-]+", "_", value.strip())
    slug = re.sub(r"_+", "_", slug).strip("._-/")
    return slug or "starter_world"


def ron_string(value: str) -> str:
    return json.dumps(value)


def ron_vec(values: list[float | int]) -> str:
    return "[" + ", ".join(f"{value:.1f}" if isinstance(value, float) else str(value) for value in values) + "]"


def hash_text(text: str) -> str:
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def world_generate_diagnostic(code: str, message: str, path: str = "world_generate") -> dict[str, Any]:
    return {
        "code": code,
        "severity": "error",
        "message": message,
        "path": path,
    }


def resolve_world_generate_output(output_arg: str) -> tuple[Path, str, list[dict[str, Any]]]:
    diagnostics: list[dict[str, Any]] = []
    output_path = Path(output_arg)
    if not output_path.is_absolute():
        output_path = (REPO_ROOT / output_path).resolve()
    else:
        output_path = output_path.resolve()

    output_rel = rel_to(output_path, REPO_ROOT)
    if not project_path_safe(output_rel):
        diagnostics.append(
            world_generate_diagnostic(
                "OUTSIDE_ALLOWLIST",
                f"World generate output must stay inside the repository/project allowlist: {output_arg}",
            )
        )
        output_rel = "output"
    return output_path, output_rel, diagnostics


def starter_world_template_content(world_id: str) -> tuple[dict[str, str], list[dict[str, Any]]]:
    sector_size = 256.0
    height = 192.0
    layers = ["terrain", "gameplay", "encounters", "foliage", "audio", "nav", "quests", "debug"]
    sectors: list[dict[str, Any]] = []
    files: dict[str, str] = {}

    for x in range(-1, 2):
        for y in range(-1, 2):
            sector_id = f"{world_id}_sector_{x}_{y}"
            sector_path = f"assets/sectors/{sector_id}.ron"
            min_x = x * sector_size
            min_z = y * sector_size
            max_x = min_x + sector_size
            max_z = min_z + sector_size
            sectors.append(
                {
                    "id": sector_id,
                    "coord": [x, y],
                    "path": sector_path,
                    "bounds": {
                        "min": [min_x, 0.0, min_z],
                        "max": [max_x, height, max_z],
                    },
                }
            )
            files[sector_path] = f"""(
    schema_version: 1,
    id: {ron_string(sector_id)},
    coord: [{x}, {y}],
    bounds: (
        min: {ron_vec([min_x, 0.0, min_z])},
        max: {ron_vec([max_x, height, max_z])},
    ),
    data_layers: [{", ".join(ron_string(layer) for layer in layers)}],
    entities: [],
    navmesh: null,
    hlod: null,
)
"""

    world_path = f"assets/worlds/{world_id}.ron"
    region_min = [-sector_size, 0.0, -sector_size]
    region_max = [sector_size * 2, height, sector_size * 2]
    sector_refs = []
    for sector in sectors:
        sector_refs.append(
            f"""                (
                    id: {ron_string(sector["id"])},
                    coord: [{sector["coord"][0]}, {sector["coord"][1]}],
                    path: {ron_string(sector["path"])},
                    required_layers: [{", ".join(ron_string(layer) for layer in ["terrain", "gameplay", "encounters"])}],
                    priority: {255 if sector["coord"] == [0, 0] else 128},
                )"""
        )
    layer_entries = []
    for layer in layers:
        default_state = "active" if layer in {"gameplay", "nav"} else "loaded"
        layer_entries.append(
            f"""        (
            id: {ron_string(layer)},
            default_state: {ron_string(default_state)},
            server_authoritative: true,
        )"""
        )

    files[world_path] = f"""(
    schema_version: 1,
    id: {ron_string(world_id)},
    display_name: {ron_string(world_id.replace("_", " ").title())},
    description: "Generated starter streamed world. Safe to inspect, validate, and extend with scene patches.",
    bounds_m: (
        min: {ron_vec(region_min)},
        max: {ron_vec(region_max)},
    ),
    sector_size_m: {sector_size:.1f},
    active_window: (
        x: 3,
        y: 3,
    ),
    streaming: (
        max_activations_per_frame: 1,
        max_deactivations_per_frame: 1,
        load_latency_budget_ms: 400.0,
        crossing_hitch_budget_ms: 6.0,
        multi_source: true,
    ),
    data_layers: [
{",".join(layer_entries)}
    ],
    regions: [
        (
            id: "starter_region",
            coord: [0, 0],
            bounds_m: (
                min: {ron_vec(region_min)},
                max: {ron_vec(region_max)},
            ),
            sectors: [
{",".join(sector_refs)}
            ],
        ),
    ],
    budgets: (
        authored_objects: 0,
        visible_instanced_props: 0,
        full_ai_agents: 0,
        low_lod_agents: 0,
        memory_mb: 512.0,
    ),
)
"""

    playtest_path = f"assets/playtests/{world_id}_traversal_smoke.ron"
    files[playtest_path] = f"""(
    schema_version: 1,
    id: {ron_string(f"{world_id}_traversal_smoke")},
    display_name: "Starter World Traversal Smoke",
    description: "Generated smoke scenario for crossing the center sector and checking streaming budget assertions.",
    seed: 1,
    duration_secs: 20.0,
    setup: (
        LoadWorld: {ron_string(world_path)},
    ),
    input_script: [
        (
            at_secs: 1.0,
            action: (
                Wait: (
                    duration_secs: 2.0,
                ),
            ),
        ),
    ],
    assertions: [
        (
            name: "center_sector_active",
            check: (
                SectorActive: {ron_string(f"{world_id}_sector_0_0")},
            ),
        ),
        (
            name: "sector_load_budget",
            check: (
                BudgetWithin: (
                    metric: "sector_load_p95_ms",
                    max: 400.0,
                ),
            ),
        ),
    ],
    artifacts: ["log", "profile"],
)
"""
    assets = [
        {"id": world_id, "kind": "world", "path": world_path},
        *[
            {"id": str(sector["id"]), "kind": "sector", "path": str(sector["path"])}
            for sector in sectors
        ],
        {"id": f"{world_id}_traversal_smoke", "kind": "playtest", "path": playtest_path},
    ]
    return files, assets


def schema_name_for_generated_kind(kind: str) -> str:
    return {
        "world": "world.schema.json",
        "sector": "sector.schema.json",
        "playtest": "playtest_scenario.schema.json",
    }[kind]


def validate_generated_file(kind: str, project_path: str, content: str, diagnostics: list[dict[str, Any]]) -> None:
    try:
        parsed = load_ron_subset_from_text(content)
        schema_name = schema_name_for_generated_kind(kind)
        schema = load_json(REPO_ROOT / "docs/specs/schemas" / schema_name)
        validate_schema(schema, parsed, schema, "$")
    except ValidationError as exc:
        diagnostics.append(world_generate_diagnostic("TEMPLATE_INVALID", str(exc), project_path))


def load_ron_subset_from_text(text: str) -> Any:
    return RonSubsetParser(text).parse()


def world_generate(template: str, output: str, name: str | None, dry_run: bool) -> dict[str, Any]:
    start = time.perf_counter()
    output_path, output_rel, diagnostics = resolve_world_generate_output(output)
    template_aliases = {"starter_world", "starter_open_world", "open_world_starter", "region_grid_3x3"}
    template_id = "starter_open_world"
    if template not in template_aliases:
        diagnostics.append(
            world_generate_diagnostic(
                "UNKNOWN_TEMPLATE",
                f"Unknown world template {template!r}; expected one of {sorted(template_aliases)}",
            )
        )
    if not dry_run:
        diagnostics.append(
            world_generate_diagnostic(
                "DRY_RUN_REQUIRED",
                "Bootstrap world generate only supports --dry-run.",
            )
        )

    world_id = slug_id(name or "starter_world")
    files, assets = starter_world_template_content(world_id)
    planned_files: list[dict[str, Any]] = []
    if project_path_safe(output_rel):
        for asset_path, content in sorted(files.items()):
            kind = next(asset["kind"] for asset in assets if asset["path"] == asset_path)
            target_path = f"{output_rel}/{asset_path}" if output_rel != "." else asset_path
            validate_generated_file(kind, target_path, content, diagnostics)
            target_abs = output_path / asset_path
            planned_files.append(
                {
                    "path": target_path,
                    "kind": kind,
                    "action": "overwrite" if target_abs.exists() else "create",
                    "exists": target_abs.exists(),
                    "bytes": len(content.encode("utf-8")),
                    "hash": hash_text(content),
                    "schema": f"docs/specs/schemas/{schema_name_for_generated_kind(kind)}",
                    "content_preview": content[:240],
                }
            )

    planned_assets = [
        {
            "id": asset["id"],
            "kind": asset["kind"],
            "path": f"{output_rel}/{asset['path']}" if output_rel != "." else asset["path"],
        }
        for asset in assets
        if project_path_safe(output_rel)
    ]
    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": not any(item["severity"] == "error" for item in diagnostics),
        "dry_run": dry_run,
        "template": template_id,
        "template_version": 1,
        "name": world_id,
        "output": output_rel,
        "planned_files": planned_files,
        "planned_assets": planned_assets,
        "diagnostics": diagnostics,
        "duration_ms": duration_ms,
    }


def profile_summary_fixture(artifact_path: str) -> dict[str, Any]:
    expected = "artifacts/profiles/open_world_enemy_camp.trace"
    if artifact_path not in {expected, str(OPEN_WORLD_FIXTURE_ROOT / "profile_summary.result.json")}:
        raise ValidationError(f"unknown bootstrap profile fixture {artifact_path!r}")
    return load_schema_validated_json(
        OPEN_WORLD_FIXTURE_ROOT / "profile_summary.result.json",
        "profile_summary_result.schema.json",
    )


def playtest_result_fixture(scenario: str) -> dict[str, Any]:
    if scenario in {"smoke", "assets/playtests/smoke.ron"}:
        return load_schema_validated_json(
            DEMO_GAME_FIXTURE_ROOT / "smoke.playtest_result.json",
            "playtest_result.schema.json",
        )
    if scenario in {"fireball_hit", "assets/playtests/fireball_hit.ron"}:
        return load_schema_validated_json(
            DEMO_GAME_FIXTURE_ROOT / "fireball_hit.playtest_result.json",
            "playtest_result.schema.json",
        )
    if scenario not in {"open_world_enemy_camp", "assets/playtests/open_world_enemy_camp.ron"}:
        raise ValidationError(f"unknown bootstrap playtest fixture {scenario!r}")
    return load_schema_validated_json(
        OPEN_WORLD_FIXTURE_ROOT / "open_world_enemy_camp.playtest_result.json",
        "playtest_result.schema.json",
    )


def project_asset_root(project_root: Path) -> tuple[Path, str]:
    project_path = project_root / "aa.project.toml"
    if project_path.is_file():
        project_data = load_toml(project_path)
        engine_config = project_data.get("engine", {})
        assets_root_name = engine_config.get("assets_root", "assets")
    else:
        assets_root_name = "assets"
    return project_root / assets_root_name, assets_root_name


def load_tag_set(assets_root: Path) -> set[str]:
    tags_path = assets_root / "data/tags.ron"
    if not tags_path.is_file():
        return set()
    data = load_ron_subset(tags_path)
    if not isinstance(data, dict):
        return set()
    return {tag for tag in data.get("tags", []) if isinstance(tag, str)}


def graph_node(
    nodes: dict[str, dict[str, Any]],
    *,
    node_id: str,
    kind: str,
    label: str,
    path: str | None = None,
    metadata: dict[str, Any] | None = None,
) -> None:
    node: dict[str, Any] = {"id": node_id, "kind": kind, "label": label}
    if path is not None:
        node["path"] = path
    if metadata:
        node["metadata"] = metadata
    nodes[node_id] = node


def graph_edge(edges: list[dict[str, str]], seen: set[tuple[str, str, str]], source: str, target: str, kind: str) -> None:
    key = (source, target, kind)
    if key in seen:
        return
    seen.add(key)
    edges.append({"from": source, "to": target, "kind": kind})


def resolve_ability_asset(project_root: Path, assets_root: Path, ability_id_or_path: str) -> tuple[Path, dict[str, Any]]:
    candidate = Path(ability_id_or_path)
    if candidate.suffix:
        path = candidate if candidate.is_absolute() else project_root / candidate
        if not path.is_file():
            raise ValidationError(f"ability asset not found: {ability_id_or_path}")
        data = load_ron_subset(path)
        if not isinstance(data, dict):
            raise ValidationError(f"ability asset is not an object: {ability_id_or_path}")
        return path, data

    for path in sorted((assets_root / "abilities").glob("*.ron")):
        data = load_ron_subset(path)
        if isinstance(data, dict) and data.get("id") == ability_id_or_path:
            return path, data

    raise ValidationError(f"ability id not found: {ability_id_or_path}")


def ability_graph(project_root: Path, ability_id_or_path: str) -> dict[str, Any]:
    start = time.perf_counter()
    assets_root, _assets_root_name = project_asset_root(project_root)
    ability_path, ability = resolve_ability_asset(project_root, assets_root, ability_id_or_path)
    ability_id = str(ability.get("id", ability_path.stem))
    ability_rel = rel_to(ability_path, project_root)
    ability_node = f"ability:{ability_id}"
    nodes: dict[str, dict[str, Any]] = {}
    edges: list[dict[str, str]] = []
    seen_edges: set[tuple[str, str, str]] = set()
    diagnostics: list[dict[str, Any]] = []
    registered_tags = load_tag_set(assets_root)

    graph_node(
        nodes,
        node_id=ability_node,
        kind="ability",
        label=ability_id,
        path=ability_rel,
        metadata={"display_name": ability.get("display_name", ability_id)},
    )

    impl = ability.get("impl")
    if isinstance(impl, str) and impl:
        impl_node = f"registrar:{impl}"
        graph_node(nodes, node_id=impl_node, kind="registrar", label=impl)
        graph_edge(edges, seen_edges, ability_node, impl_node, "implements")

    tag_fields = [
        ("cooldown_tags", "cooldown_tag"),
        ("activation_tags_required", "requires_tag"),
        ("activation_tags_blocked", "blocked_by_tag"),
    ]
    for field, edge_kind in tag_fields:
        for tag in ability.get(field, []):
            if not isinstance(tag, str):
                continue
            tag_node = f"tag:{tag}"
            graph_node(nodes, node_id=tag_node, kind="tag", label=tag)
            graph_edge(edges, seen_edges, ability_node, tag_node, edge_kind)
            if registered_tags and tag not in registered_tags:
                diagnostics.append(
                    {
                        "code": "TAG_UNREGISTERED",
                        "severity": "error",
                        "message": f"Ability uses tag not registered in assets/data/tags.ron: {tag}",
                        "path": ability_rel,
                    }
                )

    cost_effect = ability.get("cost_effect")
    if isinstance(cost_effect, str):
        effect_path = project_root / cost_effect
        if effect_path.is_file():
            effect = load_ron_subset(effect_path)
            effect_id = effect.get("id", effect_path.stem) if isinstance(effect, dict) else effect_path.stem
            effect_node = f"effect:{effect_id}"
            graph_node(
                nodes,
                node_id=effect_node,
                kind="effect",
                label=str(effect_id),
                path=rel_to(effect_path, project_root),
                metadata={
                    "modifier_count": len(effect.get("modifiers", [])) if isinstance(effect, dict) else 0,
                    "duration": effect.get("duration") if isinstance(effect, dict) else None,
                },
            )
            graph_edge(edges, seen_edges, ability_node, effect_node, "costs")
            if isinstance(effect, dict):
                for tag in effect.get("granted_tags", []):
                    if not isinstance(tag, str):
                        continue
                    tag_node = f"tag:{tag}"
                    graph_node(nodes, node_id=tag_node, kind="tag", label=tag)
                    graph_edge(edges, seen_edges, effect_node, tag_node, "grants_tag")
        else:
            diagnostics.append(
                {
                    "code": "REF_MISSING",
                    "severity": "error",
                    "message": f"Ability cost_effect soft ref is missing: {cost_effect}",
                    "path": ability_rel,
                }
            )

    ability_ref = rel_to(ability_path, project_root)
    for ai_path in sorted((assets_root / "ai").glob("*.ron")):
        ai_data = load_ron_subset(ai_path)
        if not isinstance(ai_data, dict):
            continue
        combat = ai_data.get("combat", {})
        if ability_ref not in combat.get("abilities", []):
            continue
        ai_id = ai_data.get("id", ai_path.stem)
        ai_node = f"ai_profile:{ai_id}"
        graph_node(nodes, node_id=ai_node, kind="ai_profile", label=str(ai_id), path=rel_to(ai_path, project_root))
        graph_edge(edges, seen_edges, ai_node, ability_node, "used_by")

    for action_set_path in sorted((assets_root / "action_sets").glob("*.ron")):
        action_set = load_ron_subset(action_set_path)
        if not isinstance(action_set, dict):
            continue
        grants_ability = False
        for action in action_set.get("actions", []):
            grant = action.get("GrantAbilitySet") if isinstance(action, dict) else None
            if isinstance(grant, dict) and ability_ref in grant.get("abilities", []):
                grants_ability = True
        if grants_ability:
            action_set_id = action_set.get("id", action_set_path.stem)
            action_set_node = f"action_set:{action_set_id}"
            graph_node(
                nodes,
                node_id=action_set_node,
                kind="action_set",
                label=str(action_set_id),
                path=rel_to(action_set_path, project_root),
            )
            graph_edge(edges, seen_edges, action_set_node, ability_node, "used_by")

    for experience_path in sorted((assets_root / "experiences").glob("*.ron")):
        experience = load_ron_subset(experience_path)
        if not isinstance(experience, dict):
            continue
        grants_ability = False
        for action in experience.get("actions", []):
            grant = action.get("GrantAbilitySet") if isinstance(action, dict) else None
            if isinstance(grant, dict) and ability_ref in grant.get("abilities", []):
                grants_ability = True
        if grants_ability:
            experience_id = experience.get("id", experience_path.stem)
            experience_node = f"experience:{experience_id}"
            graph_node(
                nodes,
                node_id=experience_node,
                kind="experience",
                label=str(experience_id),
                path=rel_to(experience_path, project_root),
            )
            graph_edge(edges, seen_edges, experience_node, ability_node, "used_by")

    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": not any(item["severity"] == "error" for item in diagnostics),
        "ability_id": ability_id,
        "ability_asset": ability_rel,
        "nodes": list(nodes.values()),
        "edges": edges,
        "diagnostics": diagnostics,
        "duration_ms": duration_ms,
    }


def resolve_eval_asset(eval_id_or_path: str) -> Path:
    if eval_id_or_path in {"open_world_studio_enemy_camp", "add_enemy_camp", "open_world_studio"}:
        return DEFAULT_OPEN_WORLD_EVAL
    if eval_id_or_path in {"demo_game_add_fire_ability", "add_fire_ability", "fireball_hit", "demo_game"}:
        return DEFAULT_DEMO_GAME_EVAL

    path = Path(eval_id_or_path)
    if not path.is_absolute():
        path = REPO_ROOT / path
    return path.resolve()


def config_get_diagnostic(code: str, message: str, path: str = "config") -> dict[str, Any]:
    return {
        "code": code,
        "severity": "error",
        "message": message,
        "path": path,
    }


def nested_get(data: Any, key: str) -> tuple[bool, Any]:
    current = data
    for part in key.split("."):
        if not isinstance(current, dict) or part not in current:
            return False, None
        current = current[part]
    return True, current


def config_sources(project_root: Path) -> list[tuple[Path, str]]:
    project_manifest = project_root / "aa.project.toml"
    sources: list[tuple[Path, str]] = [(project_manifest, "aa.project.toml")]
    config_root_name = "config"
    if project_manifest.is_file():
        try:
            project_data = load_toml(project_manifest)
            engine = project_data.get("engine", {}) if isinstance(project_data, dict) else {}
            if isinstance(engine.get("config_root"), str):
                config_root_name = engine["config_root"]
        except ValidationError:
            pass

    config_root = project_root / config_root_name
    for name in ("engine.toml", "game.toml", "input.toml", "scalability.toml"):
        sources.append((config_root / name, f"{config_root_name}/{name}"))
    return sources


def config_get(project_root: Path, key: str) -> dict[str, Any]:
    start = time.perf_counter()
    diagnostics: list[dict[str, Any]] = []
    project_rel = rel_to(project_root, REPO_ROOT)
    if not project_path_safe(project_rel):
        project_rel = "."

    for path, source in config_sources(project_root):
        if not path.is_file():
            continue
        try:
            data = load_toml(path)
        except ValidationError as exc:
            diagnostics.append(config_get_diagnostic("CONFIG_PARSE_FAILED", str(exc), source))
            continue
        found, value = nested_get(data, key)
        if found:
            duration_ms = int((time.perf_counter() - start) * 1000)
            return {
                "ok": True,
                "project": project_rel,
                "key": key,
                "value": value,
                "value_type": "null" if value is None else type(value).__name__,
                "source": source,
                "diagnostics": diagnostics,
                "duration_ms": duration_ms,
            }

    diagnostics.append(
        config_get_diagnostic(
            "CONFIG_KEY_NOT_FOUND",
            f"Config key was not found in project config sources: {key}",
            "aa.project.toml",
        )
    )
    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": False,
        "project": project_rel,
        "key": key,
        "source": "",
        "diagnostics": diagnostics,
        "duration_ms": duration_ms,
    }


def eval_suite_summary(path: Path) -> dict[str, Any]:
    suite = load_json(path)
    schema = load_json(REPO_ROOT / "docs/specs/schemas/agent_eval.schema.json")
    validate_schema(schema, suite, schema, "$")
    required_commands = sorted(
        {
            command
            for task in suite.get("tasks", [])
            for command in task.get("required_commands", [])
            if isinstance(command, str)
        }
    )
    categories = sorted(
        {
            task.get("category")
            for task in suite.get("tasks", [])
            if isinstance(task.get("category"), str)
        }
    )
    return {
        "id": suite["id"],
        "display_name": suite.get("display_name", suite["id"]),
        "description": suite.get("description", ""),
        "tier": suite.get("tier", "studio_alpha"),
        "path": rel(path),
        "task_count": len(suite.get("tasks", [])),
        "categories": categories,
        "required_commands": required_commands,
        "min_pass_rate": suite.get("min_pass_rate", 1.0),
        "max_repair_attempts": suite.get("max_repair_attempts", 0),
    }


def eval_list() -> dict[str, Any]:
    start = time.perf_counter()
    diagnostics: list[dict[str, Any]] = []
    suites: list[dict[str, Any]] = []
    for path in sorted({DEFAULT_DEMO_GAME_EVAL, DEFAULT_OPEN_WORLD_EVAL}):
        try:
            suites.append(eval_suite_summary(path))
        except ValidationError as exc:
            diagnostics.append(
                {
                    "code": "EVAL_LOAD_FAILED",
                    "severity": "error",
                    "message": str(exc),
                    "path": rel(path),
                }
            )

    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": not any(item["severity"] == "error" for item in diagnostics),
        "suites": suites,
        "diagnostics": diagnostics,
        "duration_ms": duration_ms,
    }


def command_run(command: str, args: list[str], exit_code: int, duration_ms: int, artifact: str | None = None) -> dict[str, Any]:
    item: dict[str, Any] = {
        "command": command,
        "args": args,
        "exit_code": exit_code,
        "duration_ms": duration_ms,
    }
    if artifact is not None:
        item["artifact"] = artifact
    return item


def run_eval_command(command: str, task: dict[str, Any], project_root: Path) -> tuple[dict[str, Any], str | None]:
    start = time.perf_counter()

    if command == "aa index":
        if task.get("category") == "enemy_camp":
            query = "enemy camp sector"
        elif task.get("category") == "ability":
            query = "fire ability fireball demo_game"
        else:
            query = task.get("prompt", "")
        result = query_index(REPO_ROOT, query)
        exit_code = 0 if result["ok"] and result["hits"] else 1
        duration_ms = int((time.perf_counter() - start) * 1000)
        failure = None if exit_code == 0 else "index returned no hits"
        return command_run(command, ["--query", query, "--json"], exit_code, duration_ms), failure

    if command == "aa validate":
        if not project_root.exists():
            duration_ms = int((time.perf_counter() - start) * 1000)
            return command_run(command, [str(project_root), "--format", "json"], 1, duration_ms), "project path missing"
        result = validate_project(project_root)
        duration_ms = int((time.perf_counter() - start) * 1000)
        failure = None if result["ok"] else "project validation failed"
        return command_run(command, [str(project_root), "--format", "json"], 0 if result["ok"] else 1, duration_ms), failure

    if command == "aa check":
        if not (project_root / "Cargo.toml").is_file():
            duration_ms = int((time.perf_counter() - start) * 1000)
            return command_run(command, [str(project_root), "--json"], 1, duration_ms), "Cargo.toml missing"
        result = check_project(project_root)
        duration_ms = int((time.perf_counter() - start) * 1000)
        failure = None if result["ok"] else "cargo check failed"
        return command_run(command, [str(project_root), "--json"], 0 if result["ok"] else 2, duration_ms), failure

    if command == "aa world inspect":
        world_inspect_fixture("open_world_studio")
        duration_ms = int((time.perf_counter() - start) * 1000)
        return command_run(command, ["--world", "open_world_studio", "--json"], 0, duration_ms), None

    if command == "aa world cook":
        world_cook_fixture("open_world_studio")
        duration_ms = int((time.perf_counter() - start) * 1000)
        return command_run(command, ["--world", "open_world_studio", "--verify", "--json"], 0, duration_ms), None

    if command == "aa playtest":
        scenario = "fireball_hit" if task.get("category") == "ability" else "open_world_enemy_camp"
        playtest_result_fixture(scenario)
        duration_ms = int((time.perf_counter() - start) * 1000)
        return command_run(command, ["--scenario", scenario, "--json"], 0, duration_ms), None

    if command == "aa profile summarize":
        profile_summary_fixture("artifacts/profiles/open_world_enemy_camp.trace")
        duration_ms = int((time.perf_counter() - start) * 1000)
        return command_run(command, ["artifacts/profiles/open_world_enemy_camp.trace", "--json"], 0, duration_ms), None

    if command == "aa ability graph":
        ability_id = "fireball" if task.get("category") == "ability" else "basic_melee"
        result = ability_graph(project_root, ability_id)
        duration_ms = int((time.perf_counter() - start) * 1000)
        failure = None if result["ok"] else "ability graph failed"
        return command_run(command, [ability_id, "--json"], 0 if result["ok"] else 1, duration_ms), failure

    if command == "aa scene inspect":
        scene = "examples/open_world_studio/assets/sectors/sector_0_0.ron"
        entity_id = "sector_0_0/entity_0"
        result = scene_inspect(scene, entity_id)
        duration_ms = int((time.perf_counter() - start) * 1000)
        failure = None if result["ok"] else "scene inspect failed"
        return command_run(command, [entity_id, "--scene", scene, "--json"], 0 if result["ok"] else 1, duration_ms), failure

    if command == "aa scene patch":
        scene = "examples/open_world_studio/assets/sectors/sector_0_0.ron"
        patch = "docs/specs/fixtures/open_world_studio/add_campfire.scene_patch.json"
        result = scene_patch_dry_run(scene, patch, dry_run=True)
        duration_ms = int((time.perf_counter() - start) * 1000)
        failure = None if result["ok"] else "scene patch dry-run failed"
        return command_run(command, ["--scene", scene, "--patch", patch, "--dry-run", "--json"], 0 if result["ok"] else 1, duration_ms), failure

    unsupported: set[str] = set()
    if command in unsupported:
        duration_ms = int((time.perf_counter() - start) * 1000)
        return command_run(command, ["--json"], 4, duration_ms), "command not implemented in bootstrap bridge"

    duration_ms = int((time.perf_counter() - start) * 1000)
    return command_run(command, [], 4, duration_ms), "unknown command"


def acceptance_item(name: str, passed: bool, message: str | None = None) -> dict[str, Any]:
    item: dict[str, Any] = {"name": name, "passed": passed}
    if message is not None:
        item["message"] = message
    return item


def command_passed(command_reports: list[dict[str, Any]], command: str) -> bool:
    return any(report["command"] == command and report["exit_code"] == 0 for report in command_reports)


def forbidden_path_matches(project_root: Path, forbidden: str) -> list[str]:
    matches: list[str] = []
    direct = project_root / forbidden
    if direct.exists():
        matches.append(rel_to(direct, project_root))
    for path in project_root.rglob(forbidden):
        if path == direct:
            continue
        if path.exists():
            matches.append(rel_to(path, project_root))
    return sorted(set(matches))


def profile_metric_value(metric: str) -> float | int | None:
    profile = profile_summary_fixture("artifacts/profiles/open_world_enemy_camp.trace")
    metric_map: dict[str, Any] = {
        "sector_load_p95_ms": profile["sector_streaming"]["load_latency"]["p95_ms"],
        "sector_crossing_hitch_ms": profile["sector_streaming"]["crossing_hitch_ms"],
        "frame_cpu_p95_ms": profile["frame"]["cpu"]["p95_ms"],
        "frame_gpu_p95_ms": profile["frame"]["gpu"]["p95_ms"],
        "memory_peak_mb": profile["memory"]["peak_mb"],
    }
    return metric_map.get(metric)


def evaluate_task_acceptance(
    task: dict[str, Any],
    project_root: Path,
    command_reports: list[dict[str, Any]],
) -> tuple[list[dict[str, Any]], list[str]]:
    checks: list[dict[str, Any]] = []
    failures: list[str] = []

    for expected_path in task.get("expected_files", []):
        path = REPO_ROOT / expected_path
        passed = path.is_file()
        checks.append(
            acceptance_item(
                f"ExpectedFile:{expected_path}",
                passed,
                None if passed else "expected task file is missing",
            )
        )
        if not passed:
            failures.append(f"expected file missing: {expected_path}")

    for forbidden in task.get("forbidden_paths", []):
        matches = forbidden_path_matches(project_root, forbidden)
        passed = not matches
        checks.append(
            acceptance_item(
                f"ForbiddenPathAbsent:{forbidden}",
                passed,
                None if passed else f"found forbidden paths: {', '.join(matches)}",
            )
        )
        if not passed:
            failures.append(f"forbidden path present: {forbidden}")

    for acceptance in task.get("acceptance", []):
        [(kind, value)] = acceptance.items()
        if kind == "CommandPasses":
            passed = command_passed(command_reports, value)
            checks.append(acceptance_item(f"CommandPasses:{value}", passed))
            if not passed:
                failures.append(f"acceptance command failed: {value}")
        elif kind == "PlaytestPasses":
            result = playtest_result_fixture(value)
            passed = bool(result["ok"])
            checks.append(acceptance_item(f"PlaytestPasses:{value}", passed))
            if not passed:
                failures.append(f"playtest acceptance failed: {value}")
        elif kind == "FileChanged":
            path = REPO_ROOT / value
            passed = path.is_file()
            checks.append(
                acceptance_item(
                    f"FileChanged:{value}",
                    passed,
                    "bootstrap verifies the authored file is present",
                )
            )
            if not passed:
                failures.append(f"file acceptance missing: {value}")
        elif kind == "NoWritesOutsideAllowlist":
            forbidden_failures = [
                forbidden
                for forbidden in task.get("forbidden_paths", [])
                if forbidden_path_matches(project_root, forbidden)
            ]
            passed = bool(value) and not forbidden_failures
            checks.append(
                acceptance_item(
                    "NoWritesOutsideAllowlist",
                    passed,
                    None if passed else f"forbidden paths present: {', '.join(forbidden_failures)}",
                )
            )
            if not passed:
                failures.append("writes outside allowlist")
        elif kind == "ProfileBudgetWithin":
            metric = value["metric"]
            maximum = value["max"]
            actual = profile_metric_value(metric)
            passed = actual is not None and actual <= maximum
            message = f"actual={actual} max={maximum}" if actual is not None else "metric missing"
            checks.append(acceptance_item(f"ProfileBudgetWithin:{metric}", passed, message))
            if not passed:
                failures.append(f"profile budget failed: {metric}")
        else:
            checks.append(acceptance_item(f"UnsupportedAcceptance:{kind}", False))
            failures.append(f"unsupported acceptance: {kind}")

    return checks, failures


def run_eval(eval_id_or_path: str, max_repairs: int | None = None) -> dict[str, Any]:
    start = time.perf_counter()
    started_at = dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")
    eval_asset = resolve_eval_asset(eval_id_or_path)
    suite = load_json(eval_asset)
    schema = load_json(REPO_ROOT / "docs/specs/schemas/agent_eval.schema.json")
    validate_schema(schema, suite, schema, "$")

    reports: list[dict[str, Any]] = []
    repair_attempts_total = 0
    for task in suite["tasks"]:
        task_project = REPO_ROOT / task.get("project", ".")
        command_reports: list[dict[str, Any]] = []
        failures: list[str] = []
        task_repairs = min(task.get("max_repair_attempts", 0), max_repairs if max_repairs is not None else task.get("max_repair_attempts", 0))
        repair_attempts_total += task_repairs

        for command in task["required_commands"]:
            report, failure = run_eval_command(command, task, task_project)
            command_reports.append(report)
            if failure:
                failures.append(f"{command}: {failure}")

        acceptance_checks, acceptance_failures = evaluate_task_acceptance(task, task_project, command_reports)
        failures.extend(acceptance_failures)
        passed = not failures
        task_report: dict[str, Any] = {
            "id": task["id"],
            "passed": passed,
            "repair_attempts": task_repairs,
            "commands": command_reports,
            "acceptance": acceptance_checks,
            "artifacts": {},
        }
        if failures:
            task_report["failure_reason"] = "; ".join(failures)
        reports.append(task_report)

    passed_count = sum(1 for report in reports if report["passed"])
    duration_ms = int((time.perf_counter() - start) * 1000)
    return {
        "ok": passed_count == len(reports),
        "suite": suite["id"],
        "eval_asset": rel(eval_asset),
        "started_at": started_at,
        "duration_ms": duration_ms,
        "pass_rate": passed_count / len(reports),
        "repair_attempts_total": repair_attempts_total,
        "tasks": reports,
        "artifacts": {},
    }


def cmd_validate(args: argparse.Namespace) -> int:
    project_root = Path(args.path).resolve()
    result = validate_project(project_root)

    if args.format == "json":
        print(json.dumps(result, indent=2, sort_keys=True))
    elif args.format == "sarif":
        print(json.dumps(validation_to_sarif(result), indent=2, sort_keys=True))
    else:
        for item in result["diagnostics"]:
            print(f"{item['severity']}: {item['path']}: {item['message']}", file=sys.stderr)
        if result["ok"]:
            print("ok", file=sys.stderr)

    return 0 if result["ok"] else 1


def cmd_check(args: argparse.Namespace) -> int:
    project_root = Path(args.path).resolve()
    result = check_project(project_root)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 2


def cmd_index(args: argparse.Namespace) -> int:
    project_root = Path(args.path).resolve()
    result = query_index(project_root, args.query, args.scope)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def cmd_config_get(args: argparse.Namespace) -> int:
    project_root = Path(args.project).resolve()
    result = config_get(project_root, args.key)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


def cmd_world_inspect(args: argparse.Namespace) -> int:
    try:
        result = world_inspect_fixture(args.world)
    except ValidationError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2, sort_keys=True))
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def cmd_world_cook(args: argparse.Namespace) -> int:
    try:
        result = world_cook_fixture(args.world)
    except ValidationError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2, sort_keys=True))
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def cmd_world_generate(args: argparse.Namespace) -> int:
    result = world_generate(args.template, args.output, args.name, args.dry_run)
    print(json.dumps(result, indent=2, sort_keys=True))
    if not args.dry_run:
        return 4
    return 0 if result["ok"] else 1


def cmd_profile_summarize(args: argparse.Namespace) -> int:
    try:
        result = profile_summary_fixture(args.artifact_path)
    except ValidationError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2, sort_keys=True))
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def cmd_playtest(args: argparse.Namespace) -> int:
    try:
        result = playtest_result_fixture(args.scenario)
    except ValidationError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2, sort_keys=True))
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def cmd_ability_graph(args: argparse.Namespace) -> int:
    project_root = Path(args.project).resolve()
    try:
        result = ability_graph(project_root, args.ability_id)
    except ValidationError as exc:
        result = {
            "ok": False,
            "ability_id": args.ability_id,
            "nodes": [],
            "edges": [],
            "diagnostics": [
                {
                    "code": "ABILITY_GRAPH_FAILED",
                    "severity": "error",
                    "message": str(exc),
                }
            ],
            "duration_ms": 0,
        }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


def cmd_scene_list(args: argparse.Namespace) -> int:
    result = scene_list(args.scene, args.filter)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


def cmd_scene_inspect(args: argparse.Namespace) -> int:
    result = scene_inspect(args.scene, args.entity_id)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


def cmd_scene_patch(args: argparse.Namespace) -> int:
    result = scene_patch_dry_run(args.scene, args.patch, args.dry_run)
    print(json.dumps(result, indent=2, sort_keys=True))
    if not args.dry_run:
        return 4
    return 0 if result["ok"] else 1


def cmd_eval(args: argparse.Namespace) -> int:
    try:
        result = run_eval(args.eval_id_or_path, args.max_repairs)
    except ValidationError as exc:
        result = {
            "ok": False,
            "suite": args.eval_id_or_path,
            "duration_ms": 0,
            "pass_rate": 0,
            "repair_attempts_total": 0,
            "tasks": [
                {
                    "id": "eval_load",
                    "passed": False,
                    "repair_attempts": 0,
                    "commands": [],
                    "failure_reason": str(exc),
                    "artifacts": {},
                }
            ],
            "artifacts": {},
        }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 3


def cmd_eval_list(args: argparse.Namespace) -> int:
    del args
    result = eval_list()
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Bootstrap AA CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_parser = subparsers.add_parser("validate", help="Validate root AA project metadata")
    validate_parser.add_argument("path", nargs="?", default=".", help="Project root")
    validate_parser.add_argument("--format", choices=["json", "sarif", "text"], default="json")
    validate_parser.set_defaults(func=cmd_validate)

    check_parser = subparsers.add_parser("check", help="Run cargo check and emit structured diagnostics")
    check_parser.add_argument("path", nargs="?", default=".", help="Cargo workspace or package root")
    check_parser.add_argument("--json", action="store_true", help="Accepted for parity with aa check")
    check_parser.set_defaults(func=cmd_check)

    index_parser = subparsers.add_parser("index", help="Query specs and bootstrap project metadata")
    index_parser.add_argument("path", nargs="?", default=".", help="Project root")
    index_parser.add_argument("--query", required=True, help="Search query")
    index_parser.add_argument("--scope", help="Optional project-relative path or directory to scan")
    index_parser.add_argument("--json", action="store_true", help="Accepted for parity with aa index")
    index_parser.set_defaults(func=cmd_index)

    config_parser = subparsers.add_parser("config", help="Bootstrap config commands")
    config_subparsers = config_parser.add_subparsers(dest="config_command", required=True)
    config_get_parser = config_subparsers.add_parser("get", help="Read a project config value")
    config_get_parser.add_argument("key")
    config_get_parser.add_argument("--project", default=".", help="Project root")
    config_get_parser.add_argument("--json", action="store_true")
    config_get_parser.set_defaults(func=cmd_config_get)

    world_parser = subparsers.add_parser("world", help="Bootstrap world fixture commands")
    world_subparsers = world_parser.add_subparsers(dest="world_command", required=True)
    world_inspect_parser = world_subparsers.add_parser("inspect", help="Return open-world inspect fixture")
    world_inspect_parser.add_argument("--world", required=True)
    world_inspect_parser.add_argument("--live", action="store_true")
    world_inspect_parser.add_argument("--json", action="store_true")
    world_inspect_parser.set_defaults(func=cmd_world_inspect)

    world_cook_parser = world_subparsers.add_parser("cook", help="Return open-world cook fixture")
    world_cook_parser.add_argument("--world", required=True)
    world_cook_parser.add_argument("--verify", action="store_true")
    world_cook_parser.add_argument("--json", action="store_true")
    world_cook_parser.set_defaults(func=cmd_world_cook)

    world_generate_parser = world_subparsers.add_parser("generate", help="Plan starter streamed world assets")
    world_generate_parser.add_argument("--template", required=True)
    world_generate_parser.add_argument("--output", required=True)
    world_generate_parser.add_argument("--name")
    world_generate_parser.add_argument("--dry-run", action="store_true")
    world_generate_parser.add_argument("--json", action="store_true")
    world_generate_parser.set_defaults(func=cmd_world_generate)

    profile_parser = subparsers.add_parser("profile", help="Bootstrap profile fixture commands")
    profile_subparsers = profile_parser.add_subparsers(dest="profile_command", required=True)
    profile_summary_parser = profile_subparsers.add_parser("summarize", help="Return profile summary fixture")
    profile_summary_parser.add_argument("artifact_path")
    profile_summary_parser.add_argument("--json", action="store_true")
    profile_summary_parser.set_defaults(func=cmd_profile_summarize)

    playtest_parser = subparsers.add_parser("playtest", help="Bootstrap playtest fixture command")
    playtest_parser.add_argument("--scenario", required=True)
    playtest_parser.add_argument("--duration")
    playtest_parser.add_argument("--headless", action="store_true")
    playtest_parser.add_argument("--json", action="store_true")
    playtest_parser.set_defaults(func=cmd_playtest)

    ability_parser = subparsers.add_parser("ability", help="Bootstrap ability fixture commands")
    ability_subparsers = ability_parser.add_subparsers(dest="ability_command", required=True)
    ability_graph_parser = ability_subparsers.add_parser("graph", help="Return an ability dependency graph")
    ability_graph_parser.add_argument("ability_id")
    ability_graph_parser.add_argument("--project", default="examples/open_world_studio", help="Project root")
    ability_graph_parser.add_argument("--json", action="store_true")
    ability_graph_parser.set_defaults(func=cmd_ability_graph)

    scene_parser = subparsers.add_parser("scene", help="Bootstrap scene fixture commands")
    scene_subparsers = scene_parser.add_subparsers(dest="scene_command", required=True)
    scene_list_parser = scene_subparsers.add_parser("list", help="List stable entity ids in a scene-like asset")
    scene_list_parser.add_argument("--scene", required=True)
    scene_list_parser.add_argument("--filter")
    scene_list_parser.add_argument("--json", action="store_true")
    scene_list_parser.set_defaults(func=cmd_scene_list)

    scene_inspect_parser = scene_subparsers.add_parser("inspect", help="Inspect a stable entity id in a scene-like asset")
    scene_inspect_parser.add_argument("entity_id")
    scene_inspect_parser.add_argument("--scene", required=True)
    scene_inspect_parser.add_argument("--json", action="store_true")
    scene_inspect_parser.set_defaults(func=cmd_scene_inspect)

    scene_patch_parser = scene_subparsers.add_parser("patch", help="Dry-run a validated scene patch")
    scene_patch_parser.add_argument("--scene", required=True)
    scene_patch_parser.add_argument("--patch", required=True)
    scene_patch_parser.add_argument("--dry-run", action="store_true")
    scene_patch_parser.add_argument("--json", action="store_true")
    scene_patch_parser.set_defaults(func=cmd_scene_patch)

    eval_parser = subparsers.add_parser("eval", help="Run bootstrap eval reports")
    eval_subparsers = eval_parser.add_subparsers(dest="eval_command", required=True)
    eval_list_parser = eval_subparsers.add_parser("list", help="List available eval fixtures")
    eval_list_parser.add_argument("--json", action="store_true", help="Accepted for parity with aa eval list")
    eval_list_parser.set_defaults(func=cmd_eval_list)

    eval_run_parser = eval_subparsers.add_parser("run", help="Run an eval fixture")
    eval_run_parser.add_argument("eval_id_or_path", help="Eval id alias or project-relative eval JSON path")
    eval_run_parser.add_argument("--max-repairs", type=int, default=None)
    eval_run_parser.add_argument("--json", action="store_true", help="Accepted for parity with aa eval run")
    eval_run_parser.set_defaults(func=cmd_eval)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
