use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::exit_codes::ExitCode;

#[derive(Debug, Serialize)]
struct SceneEntitySummary {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefab: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    layers: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct SceneDiagnostic {
    code: String,
    severity: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

/// List entities in a sector/prefab scene RON file.
pub fn list(scene: &str, filter: Option<&str>, json: bool) -> ExitCode {
    let started = Instant::now();
    let (_scene_path, scene_rel, kind, mut entities, diagnostics) = load_scene(scene);
    if let Some(filter_text) = filter {
        let lowered = filter_text.to_lowercase();
        entities.retain(|e| {
            e.id.to_lowercase().contains(&lowered)
                || e.name.as_ref().is_some_and(|n| n.to_lowercase().contains(&lowered))
                || e.prefab.as_ref().is_some_and(|p| p.to_lowercase().contains(&lowered))
        });
    }
    let ok = !diagnostics.iter().any(|d| d.severity == "error");
    let result = serde_json::json!({
        "ok": ok,
        "scene": scene_rel,
        "kind": kind,
        "entity_count": entities.len(),
        "entities": entities.iter().map(|e| SceneEntitySummary {
            id: e.id.clone(),
            name: e.name.clone(),
            prefab: e.prefab.clone(),
            layers: e.layers.clone(),
        }).collect::<Vec<_>>(),
        "diagnostics": diagnostics,
        "duration_ms": started.elapsed().as_millis(),
    });
    emit(result, json, ok)
}

/// Inspect a single entity inside a scene RON file.
pub fn inspect(scene: &str, entity_id: &str, json: bool) -> ExitCode {
    let started = Instant::now();
    let (_scene_path, scene_rel, kind, entities, mut diagnostics) = load_scene(scene);
    let found = entities.into_iter().find(|e| e.id == entity_id);
    if found.is_none() {
        diagnostics.push(SceneDiagnostic {
            code: "ENTITY_NOT_FOUND".into(),
            severity: "error".into(),
            message: format!("Scene entity id was not found: {entity_id}"),
            path: Some(scene_rel.clone()),
        });
    }
    let ok = found.is_some() && !diagnostics.iter().any(|d| d.severity == "error");
    let mut result = serde_json::json!({
        "ok": ok,
        "scene": scene_rel,
        "kind": kind,
        "entity_id": entity_id,
        "diagnostics": diagnostics,
        "duration_ms": started.elapsed().as_millis(),
    });
    if let Some(entity) = found {
        result["entity"] = serde_json::json!({
            "id": entity.id,
            "name": entity.name,
            "prefab": entity.prefab,
            "layers": entity.layers,
        });
    }
    emit(result, json, ok)
}

/// Validate a scene patch JSON against a scene target (dry-run only).
pub fn patch(scene: &str, patch: &str, dry_run: bool, json: bool) -> ExitCode {
    let started = Instant::now();
    let repo = find_workspace_root();
    let scene_path = resolve_path(&repo, scene);
    let patch_path = resolve_path(&repo, patch);
    let (project_root, scene_project_path) = infer_project_root_from_scene(&scene_path);
    let patch_diag_path = rel_path(&project_root, &patch_path);
    let mut diagnostics = Vec::new();

    if !dry_run {
        diagnostics.push(SceneDiagnostic {
            code: "DRY_RUN_REQUIRED".into(),
            severity: "error".into(),
            message: "Scene patch only supports --dry-run.".into(),
            path: Some(scene_project_path.clone()),
        });
    }
    if !scene_path.is_file() {
        diagnostics.push(SceneDiagnostic {
            code: "FILE_MISSING".into(),
            severity: "error".into(),
            message: format!("Scene target does not exist: {scene_project_path}"),
            path: Some(scene_project_path.clone()),
        });
    }

    let patch_text = std::fs::read_to_string(&patch_path).unwrap_or_default();
    let patch_value: Value = serde_json::from_str(&patch_text).unwrap_or(Value::Null);
    let patch_id = patch_value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let mut target = patch_value
        .pointer("/target/path")
        .and_then(|v| v.as_str())
        .unwrap_or(&scene_project_path)
        .to_string();

    if !project_path_safe(&target) {
        diagnostics.push(SceneDiagnostic {
            code: "SCENE_PATCH_INVALID".into(),
            severity: "error".into(),
            message: format!("Patch target path is outside the project allowlist: {target}"),
            path: Some(patch_diag_path.clone()),
        });
    }
    if target != scene_project_path {
        diagnostics.push(SceneDiagnostic {
            code: "TARGET_MISMATCH".into(),
            severity: "error".into(),
            message: format!("Patch target {target} does not match --scene {scene_project_path}"),
            path: Some(patch_diag_path.clone()),
        });
    } else {
        target = scene_project_path.clone();
    }

    let mut affected_files = vec![target.clone()];
    let mut affected_entities = Vec::new();
    let mut op_reports = Vec::new();
    if let Some(ops) = patch_value.get("ops").and_then(|v| v.as_array()) {
        for (index, op) in ops.iter().enumerate() {
            if let Some(obj) = op.as_object()
                && let Some((kind, value)) = obj.iter().next()
            {
                let entity_id = value
                    .get("entity_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                affected_entities.push(entity_id.to_string());
                let mut op_files = Vec::new();
                if kind == "InstantiatePrefab"
                    && let Some(prefab) = value.get("prefab").and_then(|v| v.as_str())
                {
                    op_files.push(prefab.to_string());
                    affected_files.push(prefab.to_string());
                    if !project_path_safe(prefab) {
                        diagnostics.push(SceneDiagnostic {
                            code: "SCENE_PATCH_INVALID".into(),
                            severity: "error".into(),
                            message: format!("Patch op path is outside the project allowlist: {prefab}"),
                            path: Some(patch_diag_path.clone()),
                        });
                    } else if !project_root.join(prefab).is_file() {
                        diagnostics.push(SceneDiagnostic {
                            code: "REF_MISSING".into(),
                            severity: "error".into(),
                            message: format!("Patch op referenced file does not exist: {prefab}"),
                            path: Some(patch_diag_path.clone()),
                        });
                    }
                }
                op_reports.push(serde_json::json!({
                    "index": index,
                    "kind": kind,
                    "entity_id": entity_id,
                    "affected_files": op_files,
                }));
            }
        }
    }

    affected_files.sort();
    affected_files.dedup();
    affected_entities.sort();
    affected_entities.dedup();

    let mut token_parts = vec![patch_id.to_string(), target.clone()];
    token_parts.extend(affected_files.clone());
    token_parts.extend(affected_entities.clone());
    let token_input = token_parts.join("|");
    let undo_token = format!("undo:dry-run:{}", sha256_prefix16(&token_input));

    let ok = !diagnostics.iter().any(|d| d.severity == "error");
    let result = serde_json::json!({
        "ok": ok,
        "dry_run": dry_run,
        "patch_id": patch_id,
        "target": target,
        "affected_files": affected_files,
        "affected_entities": affected_entities,
        "ops": op_reports,
        "undo_token": undo_token,
        "diagnostics": diagnostics,
        "duration_ms": started.elapsed().as_millis(),
    });
    emit(result, json, ok)
}

struct SceneEntity {
    id: String,
    name: Option<String>,
    prefab: Option<String>,
    layers: Option<Vec<String>>,
}

fn load_scene(scene_arg: &str) -> (PathBuf, String, String, Vec<SceneEntity>, Vec<SceneDiagnostic>) {
    let repo = find_workspace_root();
    let scene_path = resolve_path(&repo, scene_arg);
    let scene_rel = rel_path(&repo, &scene_path);
    let mut diagnostics = Vec::new();
    if !scene_path.is_file() {
        diagnostics.push(SceneDiagnostic {
            code: "SCENE_READ_FAILED".into(),
            severity: "error".into(),
            message: format!("Scene file not found: {scene_rel}"),
            path: Some(scene_rel.clone()),
        });
        return (scene_path, scene_rel, "scene".into(), Vec::new(), diagnostics);
    }

    let text = std::fs::read_to_string(&scene_path).unwrap_or_default();
    let data: SectorRon = match ron::from_str(&text) {
        Ok(d) => d,
        Err(e) => {
            diagnostics.push(SceneDiagnostic {
                code: "SCENE_READ_FAILED".into(),
                severity: "error".into(),
                message: format!("RON parse error: {e}"),
                path: Some(scene_rel.clone()),
            });
            return (scene_path, scene_rel, "sector".into(), Vec::new(), diagnostics);
        }
    };

    let root_id = data.id;
    let layers = data.data_layers.clone();
    let entities = data
        .entities
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let entity_id = format!("{root_id}/entity_{index}");
            SceneEntity {
                id: entity_id,
                name: Path::new(&item.prefab)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string()),
                prefab: Some(item.prefab.clone()),
                layers: Some(layers.clone()),
            }
        })
        .collect();
    (scene_path, scene_rel, "sector".into(), entities, diagnostics)
}

#[derive(Debug, Deserialize)]
#[serde(rename = "SectorDescriptor")]
struct SectorRon {
    id: String,
    #[serde(default)]
    data_layers: Vec<String>,
    #[serde(default)]
    entities: Vec<SectorEntityRon>,
}

#[derive(Debug, Deserialize)]
struct SectorEntityRon {
    prefab: String,
    #[serde(default)]
    #[allow(dead_code)]
    transform: SectorTransformRon,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct SectorTransformRon {
    #[serde(default)]
    translation: (f32, f32, f32),
    #[serde(default)]
    rotation_y_degrees: f32,
    #[serde(default)]
    scale: (f32, f32, f32),
}

fn emit(result: Value, json: bool, ok: bool) -> ExitCode {
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    }
    if ok {
        ExitCode::Success
    } else {
        ExitCode::ValidationFailed
    }
}

fn resolve_path(repo: &Path, arg: &str) -> PathBuf {
    let path = PathBuf::from(arg);
    if path.is_absolute() {
        path
    } else {
        repo.join(path)
    }
}

fn find_workspace_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("Cargo.toml").is_file()
            && std::fs::read_to_string(dir.join("Cargo.toml"))
                .map(|t| t.contains("[workspace]"))
                .unwrap_or(false)
        {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

fn rel_path(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn infer_project_root_from_scene(scene_path: &Path) -> (PathBuf, String) {
    let resolved = scene_path
        .canonicalize()
        .unwrap_or_else(|_| scene_path.to_path_buf());
    let normalized = resolved.to_string_lossy().replace('\\', "/");
    if let Some(idx) = normalized.find("/assets/") {
        let root = PathBuf::from(&normalized[..idx]);
        let rel = normalized[idx + 1..].to_string();
        return (root, rel);
    }
    let parent = resolved.parent().unwrap_or(&resolved).to_path_buf();
    let name = resolved
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "scene".into());
    (parent, name)
}

fn project_path_safe(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.split('/').any(|part| part == "..")
}

fn sha256_prefix16(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(input.as_bytes());
    digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()[..16]
        .to_string()
}
