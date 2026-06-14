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
    expected_files: Vec<String>,
    #[serde(default)]
    forbidden_paths: Vec<String>,
    #[serde(default)]
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
        let (acceptance_checks, acceptance_failures) =
            evaluate_task_acceptance(task, &project_root, &repo, &command_reports);
        failures.extend(acceptance_failures);
        let passed = failures.is_empty();
        let mut report = serde_json::json!({
            "id": task.id,
            "passed": passed,
            "repair_attempts": 0,
            "commands": command_reports,
            "acceptance": acceptance_checks,
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

fn eval_index_query(task: &EvalTask) -> &'static str {
    if task.category == "enemy_camp" {
        "enemy camp sector"
    } else if task.category == "ability" && task.project.contains("open_world_studio") {
        "elemental ranged ability basic_ranged_attack"
    } else if task.category == "ability" {
        "fire ability fireball demo_game"
    } else {
        "open world"
    }
}

fn eval_playtest_target(task: &EvalTask) -> (&str, &'static str) {
    if task.project.contains("open_world_studio") {
        ("examples/open_world_studio", "open_world_enemy_camp")
    } else if task.category == "ability" {
        ("examples/demo_game", "fireball_hit")
    } else {
        (task.project.as_str(), "open_world_enemy_camp")
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
            let query = eval_index_query(task);
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
            let (project, scenario) = eval_playtest_target(task);
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

fn evaluate_task_acceptance(
    task: &EvalTask,
    project_root: &Path,
    repo: &Path,
    command_reports: &[serde_json::Value],
) -> (Vec<serde_json::Value>, Vec<String>) {
    let mut checks = Vec::new();
    let mut failures = Vec::new();

    for expected_path in &task.expected_files {
        let path = repo.join(expected_path);
        let passed = path.is_file();
        checks.push(acceptance_item(
            format!("ExpectedFile:{expected_path}"),
            passed,
            if passed {
                None
            } else {
                Some("expected task file is missing".into())
            },
        ));
        if !passed {
            failures.push(format!("expected file missing: {expected_path}"));
        }
    }

    for forbidden in &task.forbidden_paths {
        let matches = forbidden_path_matches(project_root, forbidden);
        let passed = matches.is_empty();
        checks.push(acceptance_item(
            format!("ForbiddenPathAbsent:{forbidden}"),
            passed,
            if passed {
                None
            } else {
                Some(format!("found forbidden paths: {}", matches.join(", ")))
            },
        ));
        if !passed {
            failures.push(format!("forbidden path present: {forbidden}"));
        }
    }

    for acceptance in &task.acceptance {
        let Some((kind, value)) = acceptance.iter().next() else {
            continue;
        };
        match kind.as_str() {
            "CommandPasses" => {
                let command = value.as_str().unwrap_or_default();
                let passed = command_passed(command_reports, command);
                checks.push(acceptance_item(format!("CommandPasses:{command}"), passed, None));
                if !passed {
                    failures.push(format!("acceptance command failed: {command}"));
                }
            }
            "PlaytestPasses" => {
                let scenario = value.as_str().unwrap_or_default();
                let passed = playtest_passes(project_root, scenario);
                checks.push(acceptance_item(format!("PlaytestPasses:{scenario}"), passed, None));
                if !passed {
                    failures.push(format!("playtest acceptance failed: {scenario}"));
                }
            }
            "FileChanged" => {
                let path_value = value.as_str().unwrap_or_default();
                let path = repo.join(path_value);
                let passed = path.is_file();
                checks.push(acceptance_item(
                    format!("FileChanged:{path_value}"),
                    passed,
                    Some("eval verifies the authored file is present".into()),
                ));
                if !passed {
                    failures.push(format!("file acceptance missing: {path_value}"));
                }
            }
            "NoWritesOutsideAllowlist" => {
                let forbidden_failures: Vec<String> = task
                    .forbidden_paths
                    .iter()
                    .filter(|f| !forbidden_path_matches(project_root, f).is_empty())
                    .cloned()
                    .collect();
                let passed = value.as_bool().unwrap_or(false) && forbidden_failures.is_empty();
                checks.push(acceptance_item(
                    "NoWritesOutsideAllowlist".into(),
                    passed,
                    if passed {
                        None
                    } else {
                        Some(format!(
                            "forbidden paths present: {}",
                            forbidden_failures.join(", ")
                        ))
                    },
                ));
                if !passed {
                    failures.push("writes outside allowlist".into());
                }
            }
            "ProfileBudgetWithin" => {
                let metric = value
                    .get("metric")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let maximum = value
                    .get("max")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(f64::MAX);
                let actual = profile_metric_value(repo, project_root, metric);
                let passed = actual.is_some_and(|a| a <= maximum);
                let message = actual
                    .map(|a| format!("actual={a} max={maximum}"))
                    .unwrap_or_else(|| "metric missing".into());
                checks.push(acceptance_item(
                    format!("ProfileBudgetWithin:{metric}"),
                    passed,
                    Some(message),
                ));
                if !passed {
                    failures.push(format!("profile budget failed: {metric}"));
                }
            }
            other => {
                checks.push(acceptance_item(format!("UnsupportedAcceptance:{other}"), false, None));
                failures.push(format!("unsupported acceptance: {other}"));
            }
        }
    }

    (checks, failures)
}

fn acceptance_item(name: String, passed: bool, message: Option<String>) -> serde_json::Value {
    let mut item = serde_json::json!({
        "name": name,
        "passed": passed,
    });
    if let Some(message) = message {
        item["message"] = Value::String(message);
    }
    item
}

fn command_passed(command_reports: &[serde_json::Value], command: &str) -> bool {
    command_reports.iter().any(|report| {
        report.get("command").and_then(|v| v.as_str()) == Some(command)
            && report.get("exit_code").and_then(|v| v.as_i64()) == Some(0)
    })
}

fn forbidden_path_matches(project_root: &Path, forbidden: &str) -> Vec<String> {
    let mut matches = Vec::new();
    let direct = project_root.join(forbidden);
    if direct.exists() {
        matches.push(rel_path(project_root, &direct));
    }
    for entry in walkdir::WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == direct {
            continue;
        }
        if path
            .components()
            .any(|component| component.as_os_str() == forbidden)
            && path.exists()
        {
            matches.push(rel_path(project_root, path));
        }
    }
    matches.sort();
    matches.dedup();
    matches
}

fn playtest_passes(project_root: &Path, _scenario: &str) -> bool {
    let report_path = project_root.join("playtest_report.json");
    let Ok(text) = std::fs::read_to_string(&report_path) else {
        return false;
    };
    let Ok(report) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    report.get("ok").and_then(|v| v.as_bool()).unwrap_or(false)
}

fn profile_metric_value(repo: &Path, project_root: &Path, metric: &str) -> Option<f64> {
    let trace = project_root.join("artifacts/profiles/open_world_enemy_camp.trace");
    let summary = aa_world_stream::summarize_trace(&trace, repo);
    match metric {
        "sector_load_p95_ms" => Some(summary.sector_streaming.load_latency.p95_ms as f64),
        "sector_crossing_hitch_ms" => Some(summary.sector_streaming.crossing_hitch_ms as f64),
        "frame_cpu_p95_ms" => Some(summary.frame.cpu.p95_ms as f64),
        "frame_gpu_p95_ms" => Some(summary.frame.gpu.p95_ms as f64),
        "memory_peak_mb" => Some(summary.memory.peak_mb as f64),
        _ => None,
    }
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
