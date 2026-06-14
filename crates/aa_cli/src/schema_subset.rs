//! Small JSON Schema subset used by AA spec bootstrap tools and `aa validate`.

use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaValidationError {
    pub message: String,
}

impl std::fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SchemaValidationError {}

fn is_project_relative_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.contains("..") && !path.contains('\\')
}

fn matches_string_pattern(pattern: &str, value: &str) -> Result<bool, SchemaValidationError> {
    if pattern.contains("(?!") || pattern.contains("(?<") {
        return Ok(match pattern {
            "^(?!/)(?!.*\\.\\.)(?!.*\\\\).+$" => is_project_relative_path(value),
            "^(?!/)(?!.*\\\\).+$" => {
                !value.is_empty() && !value.starts_with('/') && !value.contains('\\')
            }
            "^$|^(?!/)(?!.*\\.\\.)(?!.*\\\\).+$" => {
                value.is_empty() || is_project_relative_path(value)
            }
            other => {
                return Err(SchemaValidationError {
                    message: format!("unsupported look-around pattern {other:?}"),
                });
            }
        });
    }

    let regex = Regex::new(pattern).map_err(|e| SchemaValidationError {
        message: format!("invalid schema pattern {pattern:?}: {e}"),
    })?;
    Ok(regex.is_match(value))
}

pub fn load_json(path: &Path) -> Result<Value, SchemaValidationError> {
    let text = std::fs::read_to_string(path).map_err(|e| SchemaValidationError {
        message: format!("{}: {e}", path.display()),
    })?;
    serde_json::from_str(&text).map_err(|e| SchemaValidationError {
        message: format!("{}: invalid JSON: {e}", path.display()),
    })
}

fn resolve_ref<'a>(
    ref_path: &str,
    root_schema: &'a Value,
) -> Result<&'a Value, SchemaValidationError> {
    if !ref_path.starts_with("#/") {
        return Err(SchemaValidationError {
            message: format!("unsupported $ref {ref_path:?}; only local refs are supported"),
        });
    }
    let mut target = root_schema;
    for part in ref_path[2..].split('/') {
        let part = part.replace("~1", "/").replace("~0", "~");
        target = target.get(&part).ok_or_else(|| SchemaValidationError {
            message: format!("unresolved $ref {ref_path:?}"),
        })?;
    }
    Ok(target)
}

fn type_matches(expected: &str, value: &Value) -> bool {
    match expected {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "integer" => value.as_i64().is_some(),
        "number" => value.is_number() && !value.is_boolean(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

fn canonical_json(value: &Value) -> Result<String, SchemaValidationError> {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map.iter().collect();
            let inner: Result<Vec<_>, _> = sorted
                .iter()
                .map(|(k, v)| canonical_json(v).map(|encoded| format!("{k:?}:{encoded}")))
                .collect();
            Ok(format!("{{{}}}", inner?.join(",")))
        }
        Value::Array(items) => {
            let inner: Result<Vec<_>, _> = items.iter().map(canonical_json).collect();
            Ok(format!("[{}]", inner?.join(",")))
        }
        other => serde_json::to_string(other).map_err(|e| SchemaValidationError {
            message: e.to_string(),
        }),
    }
}

pub fn validate_schema(
    schema: &Value,
    value: &Value,
    root_schema: &Value,
    path: &str,
) -> Result<(), SchemaValidationError> {
    if let Some(ref_path) = schema.get("$ref").and_then(Value::as_str) {
        let resolved = resolve_ref(ref_path, root_schema)?;
        return validate_schema(resolved, value, root_schema, path);
    }

    if let Some(options) = schema.get("oneOf").and_then(Value::as_array) {
        let mut failures = Vec::new();
        let mut matches = 0usize;
        for option in options {
            match validate_schema(option, value, root_schema, path) {
                Ok(()) => matches += 1,
                Err(err) => failures.push(err.message),
            }
        }
        if matches != 1 {
            let detail = failures.into_iter().take(3).collect::<Vec<_>>().join("; ");
            return Err(SchemaValidationError {
                message: format!(
                    "{path}: expected exactly one oneOf match, got {matches}. {detail}"
                ),
            });
        }
        return Ok(());
    }

    if let Some(expected) = schema.get("const")
        && value != expected
    {
        return Err(SchemaValidationError {
            message: format!("{path}: expected const {expected:?}, got {value:?}"),
        });
    }

    if let Some(values) = schema.get("enum").and_then(Value::as_array)
        && !values.iter().any(|item| item == value)
    {
        return Err(SchemaValidationError {
            message: format!("{path}: expected one of {values:?}, got {value:?}"),
        });
    }

    if let Some(expected_type) = schema.get("type") {
        match expected_type {
            Value::Array(types)
                if !types
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|item| type_matches(item, value)) =>
            {
                return Err(SchemaValidationError {
                    message: format!(
                        "{path}: expected type {types:?}, got {}",
                        json_type_name(value)
                    ),
                });
            }
            Value::String(expected) if !type_matches(expected, value) => {
                return Err(SchemaValidationError {
                    message: format!(
                        "{path}: expected type {expected:?}, got {}",
                        json_type_name(value)
                    ),
                });
            }
            _ => {}
        }
    }

    if let Some(text) = value.as_str() {
        if let Some(min_length) = schema.get("minLength").and_then(Value::as_u64)
            && (text.len() as u64) < min_length
        {
            return Err(SchemaValidationError {
                message: format!("{path}: string shorter than minLength {min_length}"),
            });
        }
        if let Some(pattern) = schema.get("pattern").and_then(Value::as_str)
            && !matches_string_pattern(pattern, text)?
        {
            return Err(SchemaValidationError {
                message: format!(
                    "{path}: string does not match pattern {pattern:?}: {text:?}"
                ),
            });
        }
    }

    if let Some(number) = value.as_f64().filter(|_| value.is_number()) {
        if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64)
            && number < minimum
        {
            return Err(SchemaValidationError {
                message: format!("{path}: {number} < minimum {minimum}"),
            });
        }
        if let Some(exclusive_minimum) = schema.get("exclusiveMinimum").and_then(Value::as_f64)
            && number <= exclusive_minimum
        {
            return Err(SchemaValidationError {
                message: format!("{path}: {number} <= exclusiveMinimum {exclusive_minimum}"),
            });
        }
        if let Some(maximum) = schema.get("maximum").and_then(Value::as_f64)
            && number > maximum
        {
            return Err(SchemaValidationError {
                message: format!("{path}: {number} > maximum {maximum}"),
            });
        }
    }

    if let Some(items) = value.as_array() {
        if let Some(min_items) = schema.get("minItems").and_then(Value::as_u64)
            && (items.len() as u64) < min_items
        {
            return Err(SchemaValidationError {
                message: format!("{path}: array shorter than minItems {min_items}"),
            });
        }
        if let Some(max_items) = schema.get("maxItems").and_then(Value::as_u64)
            && (items.len() as u64) > max_items
        {
            return Err(SchemaValidationError {
                message: format!("{path}: array longer than maxItems {max_items}"),
            });
        }
        if schema.get("uniqueItems").and_then(Value::as_bool) == Some(true) {
            let mut encoded = Vec::new();
            for item in items {
                encoded.push(canonical_json(item)?);
            }
            let unique: std::collections::HashSet<_> = encoded.iter().collect();
            if unique.len() != encoded.len() {
                return Err(SchemaValidationError {
                    message: format!("{path}: array items are not unique"),
                });
            }
        }
        if let Some(item_schema) = schema.get("items") {
            for (index, item) in items.iter().enumerate() {
                validate_schema(item_schema, item, root_schema, &format!("{path}[{index}]"))?;
            }
        }
    }

    if let Some(map) = value.as_object() {
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required.iter().filter_map(Value::as_str) {
                if !map.contains_key(key) {
                    return Err(SchemaValidationError {
                        message: format!("{path}: missing required key {key:?}"),
                    });
                }
            }
        }

        let properties = schema
            .get("properties")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if schema.get("additionalProperties").and_then(Value::as_bool) == Some(false) {
            let allowed: std::collections::HashSet<_> = properties.keys().collect();
            let extras: Vec<_> = map
                .keys()
                .filter(|key| !allowed.contains(key))
                .cloned()
                .collect();
            if !extras.is_empty() {
                return Err(SchemaValidationError {
                    message: format!("{path}: unexpected keys {extras:?}"),
                });
            }
        }

        for (key, child_schema) in properties {
            if let Some(child_value) = map.get(&key) {
                validate_schema(&child_schema, child_value, root_schema, &format!("{path}.{key}"))?;
            }
        }
    }

    Ok(())
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) if n.as_i64().is_some() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
