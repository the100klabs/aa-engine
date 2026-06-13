use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::exit_codes::ExitCode;
use crate::project::{self, ProjectError};

#[derive(Debug, Deserialize)]
struct PlaytestReport {
    ok: bool,
    scenario: String,
    duration_secs: f32,
    assertions: Vec<PlaytestAssertion>,
}

#[derive(Debug, Deserialize)]
struct PlaytestAssertion {
    name: String,
    passed: bool,
}

/// Run an automated headless playtest scenario.
pub fn run(
    path: &Path,
    scenario: &str,
    duration_secs: u32,
    json: bool,
) -> ExitCode {
    let project_root = match project::resolve_project_root(path) {
        Ok(root) => root,
        Err(ProjectError::ManifestMissing(p)) => {
            eprintln!("error: aa.project.toml not found at {}", p.display());
            return ExitCode::InvalidArgs;
        }
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::InternalError;
        }
    };

    let manifest = match project::load_manifest(&project_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::InvalidArgs;
        }
    };

    let workspace_root = find_workspace_root(&project_root);
    let report_path = project_root.join("playtest_report.json");
    let _ = std::fs::remove_file(&report_path);

    eprintln!(
        "Running playtest scenario '{scenario}' for {duration_secs}s on '{}' …",
        manifest.name
    );

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&workspace_root);
    cmd.args(["run", "-p", &manifest.name, "--quiet"]);
    cmd.env("AA_PLAYTEST", "1");
    cmd.env("AA_PLAYTEST_SCENARIO", scenario);
    cmd.env("AA_PLAYTEST_DURATION", duration_secs.to_string());

    let started = Instant::now();
    let timeout = Duration::from_secs(duration_secs as u64 + 60);

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to spawn playtest: {e}");
            return ExitCode::InternalError;
        }
    };

    if started.elapsed() > timeout {
        eprintln!("error: playtest timed out after {}s", timeout.as_secs());
        return ExitCode::InternalError;
    }

    if !status.success() {
        eprintln!("error: playtest process exited with {status}");
        return ExitCode::ValidationFailed;
    }

    let report = match std::fs::read_to_string(&report_path) {
        Ok(text) => match serde_json::from_str::<PlaytestReport>(&text) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: invalid playtest report JSON: {e}");
                return ExitCode::InternalError;
            }
        },
        Err(e) => {
            eprintln!("error: playtest report missing at {}: {e}", report_path.display());
            return ExitCode::InternalError;
        }
    };

    if json {
        if let Ok(text) = std::fs::read_to_string(&report_path) {
            println!("{text}");
        }
    } else if report.ok {
        eprintln!(
            "playtest passed: {} ({} assertions, {:.1}s)",
            report.scenario,
            report.assertions.len(),
            report.duration_secs
        );
    } else {
        for assertion in &report.assertions {
            if !assertion.passed {
                eprintln!("FAILED: {}", assertion.name);
            }
        }
        eprintln!("playtest failed: {}", report.scenario);
    }

    if report.ok {
        ExitCode::Success
    } else {
        ExitCode::ValidationFailed
    }
}

fn find_workspace_root(project_root: &Path) -> PathBuf {
    let mut dir = project_root.to_path_buf();
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
    project_root.to_path_buf()
}
