use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde::Deserialize;
use serde_json::Value;

use crate::exit_codes::ExitCode;

#[derive(Debug, Deserialize)]
struct EvalSuite {
    id: String,
    tasks: Vec<EvalTask>,
}

#[derive(Debug, Deserialize)]
struct EvalTask {
    id: String,
    #[serde(default)]
    project: String,
    #[serde(default)]
    category: String,
    required_commands: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    acceptance: Vec<serde_json::Map<String, Value>>,
}

/// List known eval suites.
pub fn list(json: bool) -> ExitCode {
    let started = Instant::now();
    let repo = find_workspace_root();
    let suites = [
        repo.join("docs/specs/fixtures/demo_game/add_fireball.eval.json"),
        repo.join("docs/specs/fixtures/open_world_studio/add_enemy_camp.eval.json"),
        repo.join("docs/specs/fixtures/open_world_studio/add_elemental_ability.eval.json"),
    ];
    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    for path in suites {
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                if let Ok(suite) = serde_json::from_str::<EvalSuite>(&text) {
                    entries.push(serde_json::json!({
                        "id": suite.id,
                        "path": rel_path(&repo, &path),
                        "task_count": suite.tasks.len(),
                    }));
                }
            }
            Err(e) => diagnostics.push(serde_json::json!({
                "code": "EVAL_LOAD_FAILED",
                "severity": "error",
                "message": e.to_string(),
                "path": rel_path(&repo, &path),
            })),
        }
    }
    let result = serde_json::json!({
        "ok": diagnostics.is_empty(),
        "suites": entries,
        "diagnostics": diagnostics,
        "duration_ms": started.elapsed().as_millis(),
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else {
        for suite in &entries {
            eprintln!("  {} ({})", suite["id"], suite["path"]);
        }
    }
    if result["ok"].as_bool().unwrap_or(false) {
        ExitCode::Success
    } else {
        ExitCode::ValidationFailed
    }
}

/// Run an eval suite against the real Rust CLI + runtime.
pub fn run_eval(eval_id_or_path: &str, json: bool) -> ExitCode {
    let started = Instant::now();
    let repo = find_workspace_root();
    let eval_path = resolve_eval_path(&repo, eval_id_or_path);
    let text = match std::fs::read_to_string(&eval_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: failed to read eval {}: {e}", eval_path.display());
            return ExitCode::InvalidArgs;
        }
    };
    let suite: EvalSuite = match serde_json::from_str(&text) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: invalid eval JSON: {e}");
            return ExitCode::InvalidArgs;
        }
    };

    let mut task_reports = Vec::new();
    for task in &suite.tasks {
        let project_root = repo.join(&task.project);
        let mut command_reports = Vec::new();
        let mut failures = Vec::new();
        for command in &task.required_commands {
            let (report, failure) = run_eval_command(command, task, &project_root, &repo);
            command_reports.push(report);
            if let Some(f) = failure {
                failures.push(format!("{command}: {f}"));
            }
        }
        let passed = failures.is_empty();
        let mut report = serde_json::json!({
            "id": task.id,
            "passed": passed,
            "repair_attempts": 0,
            "commands": command_reports,
            "acceptance": [],
            "artifacts": {},
        });
        if !passed {
            report["failure_reason"] = Value::String(failures.join("; "));
        }
        task_reports.push(report);
    }

    let passed_count = task_reports.iter().filter(|r| r["passed"].as_bool() == Some(true)).count();
    let result = serde_json::json!({
        "ok": passed_count == task_reports.len(),
        "suite": suite.id,
        "eval_asset": rel_path(&repo, &eval_path),
        "duration_ms": started.elapsed().as_millis(),
        "pass_rate": if task_reports.is_empty() { 0.0 } else { passed_count as f64 / task_reports.len() as f64 },
        "repair_attempts_total": 0,
        "tasks": task_reports,
        "artifacts": {},
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else if result["ok"].as_bool() == Some(true) {
        eprintln!("eval passed: {}", suite.id);
    } else {
        eprintln!("eval failed: {}", suite.id);
    }

    if result["ok"].as_bool() == Some(true) {
        ExitCode::Success
    } else {
        ExitCode::ValidationFailed
    }
}

fn run_eval_command(
    command: &str,
    task: &EvalTask,
    project_root: &Path,
    repo: &Path,
) -> (serde_json::Value, Option<String>) {
    let started = Instant::now();
    let (args, failure) = match command {
        "aa index" => {
            let query = if task.category == "enemy_camp" {
                "enemy camp sector"
            } else if task.category == "ability" {
                "fire ability fireball demo_game"
            } else {
                "open world"
            };
            let code = run_aa(repo, &["index", project_root.to_str().unwrap(), "--query", query, "--json"]);
            (
                vec!["--query", query, "--json"],
                if code == 0 { None } else { Some("index failed".into()) },
            )
        }
        "aa validate" => {
            let code = run_aa(repo, &["validate", project_root.to_str().unwrap(), "--format", "json"]);
            (vec!["--format", "json"], if code == 0 { None } else { Some("validate failed".into()) })
        }
        "aa check" => {
            let code = run_aa(repo, &["check", project_root.to_str().unwrap(), "--json"]);
            (vec!["--json"], if code == 0 { None } else { Some("check failed".into()) })
        }
        "aa world inspect" => {
            let code = run_aa(
                repo,
                &[
                    "world",
                    "inspect",
                    "--project",
                    "examples/open_world_studio",
                    "--world",
                    "open_world_studio",
                    "--json",
                ],
            );
            (vec!["--world", "open_world_studio", "--json"], if code == 0 { None } else { Some("world inspect failed".into()) })
        }
        "aa world cook" => {
            let code = run_aa(
                repo,
                &[
                    "world",
                    "cook",
                    "--project",
                    "examples/open_world_studio",
                    "--world",
                    "open_world_studio",
                    "--verify",
                    "--json",
                ],
            );
            (
                vec!["--world", "open_world_studio", "--verify", "--json"],
                if code == 0 { None } else { Some("world cook failed".into()) },
            )
        }
        "aa playtest" => {
            let scenario = if task.category == "ability" {
                "fireball_hit"
            } else {
                "open_world_enemy_camp"
            };
            let project = if task.category == "ability" {
                "examples/demo_game"
            } else {
                "examples/open_world_studio"
            };
            let code = run_aa(
                repo,
                &[
                    "playtest",
                    "--project",
                    project,
                    "--scenario",
                    scenario,
                    "--duration",
                    "20",
                    "--json",
                ],
            );
            (
                vec!["--scenario", scenario, "--json"],
                if code == 0 { None } else { Some("playtest failed".into()) },
            )
        }
        "aa profile summarize" => {
            let trace = "examples/open_world_studio/artifacts/profiles/open_world_enemy_camp.trace";
            let code = run_aa(repo, &["profile", "summarize", trace, "--json"]);
            (vec![trace, "--json"], if code == 0 { None } else { Some("profile summarize failed".into()) })
        }
        "aa scene inspect" => {
            let code = run_aa(
                repo,
                &[
                    "scene",
                    "inspect",
                    "sector_0_0/entity_0",
                    "--scene",
                    "examples/open_world_studio/assets/sectors/sector_0_0.ron",
                    "--json",
                ],
            );
            (
                vec!["sector_0_0/entity_0", "--scene", "examples/open_world_studio/assets/sectors/sector_0_0.ron", "--json"],
                if code == 0 { None } else { Some("scene inspect failed".into()) },
            )
        }
        "aa scene patch" => {
            let code = run_aa(
                repo,
                &[
                    "scene",
                    "patch",
                    "--scene",
                    "examples/open_world_studio/assets/sectors/sector_0_0.ron",
                    "--patch",
                    "docs/specs/fixtures/open_world_studio/add_campfire.scene_patch.json",
                    "--dry-run",
                    "--json",
                ],
            );
            (
                vec!["--dry-run", "--json"],
                if code == 0 { None } else { Some("scene patch failed".into()) },
            )
        }
        _ => (vec![], Some("unknown command".into())),
    };

    let report = serde_json::json!({
        "command": command,
        "args": args,
        "exit_code": if failure.is_none() { 0 } else { 1 },
        "duration_ms": started.elapsed().as_millis(),
    });
    (report, failure)
}

fn run_aa(repo: &Path, args: &[&str]) -> i32 {
    let status = Command::new("cargo")
        .current_dir(repo)
        .args(["run", "-p", "aa_cli", "--quiet", "--"])
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(_) => 1,
    }
}

fn resolve_eval_path(repo: &Path, id_or_path: &str) -> PathBuf {
    let direct = repo.join(id_or_path);
    if direct.is_file() {
        return direct;
    }
    let fixtures = repo.join("docs/specs/fixtures");
    for entry in walkdir::WalkDir::new(&fixtures)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(path)
            && let Ok(value) = serde_json::from_str::<Value>(&text)
            && value.get("id").and_then(|v| v.as_str()) == Some(id_or_path)
        {
            return path.to_path_buf();
        }
    }
    repo.join(format!("docs/specs/fixtures/open_world_studio/{id_or_path}.eval.json"))
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
