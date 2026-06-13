use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde::Serialize;

use crate::exit_codes::ExitCode;

#[derive(Serialize)]
struct CheckReport {
    ok: bool,
    errors: Vec<CheckError>,
    warnings: Vec<String>,
    duration_ms: u64,
}

#[derive(Serialize)]
struct CheckError {
    file: String,
    line: Option<u32>,
    message: String,
}

/// Run `cargo check` for the package at `path` (or workspace if no manifest).
pub fn run(path: &Path, json: bool) -> ExitCode {
    let started = Instant::now();
    eprintln!("Running cargo check in {} …", path.display());

    let manifest = find_cargo_manifest(path);
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    if let Some(manifest_path) = manifest {
        cmd.args(["--manifest-path", manifest_path.to_string_lossy().as_ref()]);
    } else {
        cmd.arg("--workspace");
    }

    let output = match cmd.output() {
        Ok(output) => output,
        Err(err) => {
            eprintln!("error: failed to spawn cargo: {err}");
            return ExitCode::InternalError;
        }
    };

    let ok = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let errors = parse_cargo_errors(&stderr);

    if !ok {
        eprintln!("{stderr}");
    } else {
        eprintln!("cargo check succeeded.");
    }

    if json {
        let report = CheckReport {
            ok,
            errors,
            warnings: vec![],
            duration_ms: started.elapsed().as_millis() as u64,
        };
        if let Ok(text) = serde_json::to_string_pretty(&report) {
            println!("{text}");
        }
    }

    if ok {
        ExitCode::Success
    } else {
        ExitCode::CompileFailed
    }
}

fn find_cargo_manifest(path: &Path) -> Option<std::path::PathBuf> {
    let direct = path.join("Cargo.toml");
    if direct.is_file() {
        return Some(direct);
    }
    // Game projects live under examples/ with aa.project.toml but share workspace Cargo.toml.
    if path.join("aa.project.toml").is_file()
        && let Ok(cwd) = std::env::current_dir()
    {
        let workspace = cwd.join("Cargo.toml");
        if workspace.is_file() {
            return Some(workspace);
        }
    }
    None
}

/// Best-effort parse of `error[...]:` lines from cargo stderr.
fn parse_cargo_errors(stderr: &str) -> Vec<CheckError> {
    let mut errors = Vec::new();
    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error") {
            errors.push(CheckError {
                file: String::new(),
                line: None,
                message: trimmed.to_string(),
            });
        } else if let Some(rest) = trimmed.strip_prefix("--> ")
            && let Some((file, line_part)) = rest.split_once(':')
            && let Some(last) = errors.last_mut()
        {
            last.file = file.trim().to_string();
            last.line = line_part.trim().parse().ok();
        }
    }
    errors
}
