use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;
use walkdir::WalkDir;

use crate::exit_codes::ExitCode;
use crate::project::{self, ProjectError, REQUIRED_CONFIG_FILES};

#[derive(Debug, Clone, Serialize)]
pub struct ValidationDiagnostic {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<DiagnosticSpan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticSpan {
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

#[derive(Debug, Clone)]
struct ValidationError {
    pub code: String,
    pub message: String,
    pub path: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Serialize)]
struct ValidateReport {
    ok: bool,
    error_count: usize,
    warning_count: usize,
    duration_ms: u64,
    diagnostics: Vec<ValidationDiagnostic>,
}

/// Validate project assets and configuration.
pub fn run(path: &Path, json: bool, sarif: bool) -> ExitCode {
    let started = Instant::now();
    let project_root = match project::resolve_project_root(path) {
        Ok(root) => root,
        Err(ProjectError::ManifestMissing(p)) => {
            let err = ValidationError {
                code: "PROJECT_MANIFEST_MISSING".into(),
                message: format!("aa.project.toml not found at {}", p.display()),
                path: p.to_string_lossy().into_owned(),
                line: None,
                column: None,
            };
            return finish(vec![err], 0, started, json, sarif);
        }
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::InternalError;
        }
    };

    eprintln!("Validating project at {} …", project_root.display());

    let mut errors = Vec::new();
    let warnings = 0usize;

    let manifest = match project::load_manifest(&project_root) {
        Ok(m) => m,
        Err(e) => {
            errors.push(validation_error("MANIFEST_PARSE", e.to_string(), project::manifest_path(&project_root)));
            return finish(errors, warnings, started, json, sarif);
        }
    };

    if manifest.schema_version == 0 {
        errors.push(validation_error(
            "SCHEMA_VERSION",
            "schema_version must be >= 1".to_string(),
            project::manifest_path(&project_root),
        ));
    }

    let config_dir = project_root.join(&manifest.engine.config_root);
    for file in REQUIRED_CONFIG_FILES {
        let config_path = config_dir.join(file);
        if !config_path.is_file() {
            errors.push(validation_error(
                "CONFIG_MISSING",
                format!("required config file missing: {file}"),
                config_path,
            ));
        } else if let Err(e) = std::fs::read_to_string(&config_path).and_then(|t| {
            toml::from_str::<toml::Table>(&t)
                .map(|_| ())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }) {
            errors.push(validation_error(
                "CONFIG_PARSE",
                format!("failed to parse TOML: {e}"),
                config_path,
            ));
        }
    }

    let assets_root = project_root.join(&manifest.engine.assets_root);
    if !assets_root.is_dir() {
        errors.push(validation_error(
            "ASSETS_ROOT_MISSING",
            format!("assets root not found: {}", manifest.engine.assets_root),
            assets_root,
        ));
        return finish(errors, warnings, started, json, sarif);
    }

    let mut prefab_ids: HashMap<String, PathBuf> = HashMap::new();
    for entry in WalkDir::new(&assets_root).into_iter().filter_map(Result::ok) {
        let file_path = entry.path();
        if file_path.extension().and_then(|e| e.to_str()) != Some("ron") {
            continue;
        }

        let rel = file_path
            .strip_prefix(&assets_root)
            .unwrap_or(file_path)
            .with_extension("");
        let asset_id = rel.to_string_lossy().replace('\\', "/");

        if asset_id.starts_with("prefabs/") {
            prefab_ids.insert(asset_id, file_path.to_path_buf());
        }

        let text = match std::fs::read_to_string(file_path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(validation_error(
                    "RON_READ",
                    format!("failed to read RON file: {e}"),
                    file_path,
                ));
                continue;
            }
        };

        if let Err(e) = ron::from_str::<ron::Value>(&text) {
            errors.push(ValidationError {
                code: "RON_PARSE".into(),
                message: format!("RON parse error: {e}"),
                path: file_path
                    .strip_prefix(&project_root)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .into_owned(),
                line: None,
                column: None,
            });
        }
    }

    validate_prefab_refs(&project_root, &assets_root, &prefab_ids, &mut errors);
    detect_prefab_cycles(&prefab_ids, &mut errors);

    if let Some(scene) = manifest.engine.startup_scene.as_deref() {
        let scene_path = assets_root.join(format!("{scene}.ron"));
        if !scene_path.is_file() {
            errors.push(validation_error(
                "REF_MISSING",
                format!("startup_scene '{scene}' not found"),
                scene_path,
            ));
        }
    }

    if let Some(experience) = manifest.engine.default_experience.as_deref() {
        let experience_path = assets_root.join(format!("{experience}.ron"));
        if !experience_path.is_file() {
            errors.push(validation_error(
                "REF_MISSING",
                format!("default_experience '{experience}' not found"),
                experience_path,
            ));
        }
    }

    finish(errors, warnings, started, json, sarif)
}

fn validate_prefab_refs(
    project_root: &Path,
    assets_root: &Path,
    prefab_ids: &HashMap<String, PathBuf>,
    errors: &mut Vec<ValidationError>,
) {
    for entry in WalkDir::new(assets_root).into_iter().filter_map(Result::ok) {
        let file_path = entry.path();
        if file_path.extension().and_then(|e| e.to_str()) != Some("ron") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(file_path) else {
            continue;
        };

        for (prefab_id, byte_offset) in extract_prefab_refs(&text) {
            let normalized = normalize_prefab_ref(&prefab_id);
            if prefab_ids.contains_key(&normalized) {
                continue;
            }
            let asset_ref = prefab_id.strip_prefix("assets/").unwrap_or(&prefab_id);
            if assets_root.join(asset_ref).is_file() {
                continue;
            }
            let rel = file_path
                .strip_prefix(project_root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .into_owned();
            errors.push(ValidationError {
                code: "REF_MISSING".into(),
                message: format!("prefab ref '{prefab_id}' does not resolve"),
                path: rel,
                line: line_number_for_match(&text, byte_offset),
                column: None,
            });
        }
    }
}

fn detect_prefab_cycles(prefab_ids: &HashMap<String, PathBuf>, errors: &mut Vec<ValidationError>) {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for (id, path) in prefab_ids {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let refs: Vec<String> = extract_prefab_refs(&text)
            .into_iter()
            .map(|(r, _)| normalize_prefab_ref(&r))
            .filter(|r| prefab_ids.contains_key(r))
            .collect();
        graph.insert(id.clone(), refs);
    }

    for start in graph.keys() {
        if let Some(cycle) = find_cycle(start, &graph) {
            errors.push(ValidationError {
                code: "CYCLE_PREFAB".into(),
                message: format!("cyclic prefab refs: {}", cycle.join(" -> ")),
                path: format!("assets/{start}.ron"),
                line: None,
                column: None,
            });
            break;
        }
    }
}

fn find_cycle(start: &str, graph: &HashMap<String, Vec<String>>) -> Option<Vec<String>> {
    let mut stack = vec![start.to_string()];
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut parent: HashMap<String, String> = HashMap::new();

    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        if visiting.contains(&node) {
            let mut cycle = vec![node.clone()];
            let mut cur = node.clone();
            while let Some(p) = parent.get(&cur) {
                cycle.push(p.clone());
                if p == start {
                    cycle.reverse();
                    return Some(cycle);
                }
                cur = p.clone();
            }
            return None;
        }
        visiting.insert(node.clone());
        stack.push(node.clone());
        if let Some(neighbors) = graph.get(&node) {
            for n in neighbors {
                parent.entry(n.clone()).or_insert_with(|| node.clone());
                stack.push(n.clone());
            }
        }
        visiting.remove(&node);
        visited.insert(node);
    }
    None
}

fn normalize_prefab_ref(prefab_id: &str) -> String {
    let mut id = prefab_id.trim();
    if let Some(stripped) = id.strip_prefix("assets/") {
        id = stripped;
    }
    if let Some(stripped) = id.strip_suffix(".ron") {
        id = stripped;
    }
    id.to_string()
}

/// Scan RON text for `prefab: "some/id"` soft references.
fn extract_prefab_refs(text: &str) -> Vec<(String, usize)> {
    let mut refs = Vec::new();
    let needle = "prefab:";
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(needle) {
        let start = search_from + rel;
        let after = &text[start + needle.len()..];
        let after = after.trim_start();
        if let Some(id) = parse_quoted_string(after) {
            refs.push((id, start));
        }
        search_from = start + needle.len();
    }
    refs
}

fn parse_quoted_string(s: &str) -> Option<String> {
    let s = s.trim_start();
    if !s.starts_with('"') {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    for ch in s[1..].chars() {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            c => out.push(c),
        }
    }
    None
}

fn line_number_for_match(text: &str, byte_offset: usize) -> Option<u32> {
    let prefix = text.get(..byte_offset)?;
    Some(prefix.lines().count() as u32)
}

fn validation_error(code: &str, message: String, path: impl AsRef<Path>) -> ValidationError {
    ValidationError {
        code: code.into(),
        message,
        path: path.as_ref().to_string_lossy().into_owned(),
        line: None,
        column: None,
    }
}

fn finish(
    errors: Vec<ValidationError>,
    warning_count: usize,
    started: Instant,
    json: bool,
    sarif: bool,
) -> ExitCode {
    let ok = errors.is_empty();
    let diagnostics: Vec<ValidationDiagnostic> = errors
        .iter()
        .map(|err| ValidationDiagnostic {
            code: err.code.clone(),
            severity: "error".into(),
            message: err.message.clone(),
            path: err.path.clone(),
            span: err.line.map(|line| DiagnosticSpan {
                line,
                column: err.column,
            }),
        })
        .collect();

    let report = ValidateReport {
        ok,
        error_count: errors.len(),
        warning_count,
        duration_ms: started.elapsed().as_millis() as u64,
        diagnostics,
    };

    if sarif {
        println!("{}", serde_json::to_string_pretty(&validation_to_sarif(&report)).unwrap_or_default());
    } else if json {
        if let Ok(text) = serde_json::to_string_pretty(&report) {
            println!("{text}");
        }
    } else if ok {
        eprintln!(
            "validation passed ({} ms)",
            report.duration_ms
        );
    } else {
        for diag in &report.diagnostics {
            eprintln!("{}: {} — {}", diag.code, diag.path, diag.message);
        }
        eprintln!(
            "validation failed: {} error(s), {} warning(s) ({} ms)",
            report.error_count, report.warning_count, report.duration_ms
        );
    }

    if ok {
        ExitCode::Success
    } else {
        ExitCode::ValidationFailed
    }
}

fn validation_to_sarif(report: &ValidateReport) -> serde_json::Value {
    let results: Vec<serde_json::Value> = report
        .diagnostics
        .iter()
        .map(|diag| {
            serde_json::json!({
                "ruleId": diag.code,
                "level": diag.severity,
                "message": { "text": diag.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": diag.path },
                        "region": diag.span.as_ref().map(|span| serde_json::json!({ "startLine": span.line }))
                    }
                }]
            })
        })
        .collect();
    serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [{
            "tool": { "driver": { "name": "aa_cli", "version": "0.1.0" } },
            "results": results,
        }]
    })
}
