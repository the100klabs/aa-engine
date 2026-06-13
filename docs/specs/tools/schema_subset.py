"""Small JSON Schema subset used by AA spec bootstrap tools."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any


class ValidationError(Exception):
    pass


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValidationError(f"{path}: invalid JSON: {exc}") from exc


def resolve_ref(ref: str, root_schema: dict[str, Any]) -> Any:
    if not ref.startswith("#/"):
        raise ValidationError(f"unsupported $ref {ref!r}; only local refs are supported")

    target: Any = root_schema
    for part in ref[2:].split("/"):
        part = part.replace("~1", "/").replace("~0", "~")
        try:
            target = target[part]
        except (KeyError, TypeError) as exc:
            raise ValidationError(f"unresolved $ref {ref!r}") from exc
    return target


def type_matches(expected: str, value: Any) -> bool:
    if expected == "object":
        return isinstance(value, dict)
    if expected == "array":
        return isinstance(value, list)
    if expected == "string":
        return isinstance(value, str)
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return (isinstance(value, int) or isinstance(value, float)) and not isinstance(value, bool)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "null":
        return value is None
    return False


def validate_schema(schema: dict[str, Any], value: Any, root_schema: dict[str, Any], path: str) -> None:
    if "$ref" in schema:
        validate_schema(resolve_ref(schema["$ref"], root_schema), value, root_schema, path)
        return

    if "oneOf" in schema:
        failures: list[str] = []
        matches = 0
        for option in schema["oneOf"]:
            try:
                validate_schema(option, value, root_schema, path)
            except ValidationError as exc:
                failures.append(str(exc))
            else:
                matches += 1
        if matches != 1:
            detail = "; ".join(failures[:3])
            raise ValidationError(f"{path}: expected exactly one oneOf match, got {matches}. {detail}")
        return

    if "const" in schema and value != schema["const"]:
        raise ValidationError(f"{path}: expected const {schema['const']!r}, got {value!r}")

    if "enum" in schema and value not in schema["enum"]:
        raise ValidationError(f"{path}: expected one of {schema['enum']!r}, got {value!r}")

    expected_type = schema.get("type")
    if expected_type:
        if isinstance(expected_type, list):
            if not any(type_matches(item, value) for item in expected_type):
                raise ValidationError(f"{path}: expected type {expected_type!r}, got {type(value).__name__}")
        elif not type_matches(expected_type, value):
            raise ValidationError(f"{path}: expected type {expected_type!r}, got {type(value).__name__}")

    if isinstance(value, str):
        if "minLength" in schema and len(value) < schema["minLength"]:
            raise ValidationError(f"{path}: string shorter than minLength {schema['minLength']}")
        if "pattern" in schema and re.search(schema["pattern"], value) is None:
            raise ValidationError(f"{path}: string does not match pattern {schema['pattern']!r}: {value!r}")

    if isinstance(value, (int, float)) and not isinstance(value, bool):
        if "minimum" in schema and value < schema["minimum"]:
            raise ValidationError(f"{path}: {value} < minimum {schema['minimum']}")
        if "exclusiveMinimum" in schema and value <= schema["exclusiveMinimum"]:
            raise ValidationError(f"{path}: {value} <= exclusiveMinimum {schema['exclusiveMinimum']}")
        if "maximum" in schema and value > schema["maximum"]:
            raise ValidationError(f"{path}: {value} > maximum {schema['maximum']}")

    if isinstance(value, list):
        if "minItems" in schema and len(value) < schema["minItems"]:
            raise ValidationError(f"{path}: array shorter than minItems {schema['minItems']}")
        if "maxItems" in schema and len(value) > schema["maxItems"]:
            raise ValidationError(f"{path}: array longer than maxItems {schema['maxItems']}")
        if schema.get("uniqueItems"):
            encoded = [json.dumps(item, sort_keys=True) for item in value]
            if len(encoded) != len(set(encoded)):
                raise ValidationError(f"{path}: array items are not unique")
        if "items" in schema:
            for index, item in enumerate(value):
                validate_schema(schema["items"], item, root_schema, f"{path}[{index}]")

    if isinstance(value, dict):
        required = schema.get("required", [])
        for key in required:
            if key not in value:
                raise ValidationError(f"{path}: missing required key {key!r}")

        properties = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            extras = set(value) - set(properties)
            if extras:
                raise ValidationError(f"{path}: unexpected keys {sorted(extras)!r}")

        for key, child_schema in properties.items():
            if key in value:
                validate_schema(child_schema, value[key], root_schema, f"{path}.{key}")


def project_relative(path: str) -> bool:
    return bool(path) and not path.startswith("/") and ".." not in path and "\\" not in path


def walk_project_paths(value: Any, path: str) -> None:
    path_keys = {"fixture", "schema", "path", "artifact", "world_asset", "project", "final_diff"}
    if isinstance(value, dict):
        for key, child in value.items():
            child_path = f"{path}.{key}" if path else key
            if key in path_keys or key.endswith("_asset"):
                if isinstance(child, str) and not project_relative(child):
                    raise ValidationError(f"{child_path}: expected project-relative path, got {child!r}")
            walk_project_paths(child, child_path)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk_project_paths(child, f"{path}[{index}]")
