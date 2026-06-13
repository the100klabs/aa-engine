use std::path::Path;
use std::process::Command;

use crate::exit_codes::ExitCode;
use crate::project::{self, ProjectError};

/// Launch the game binary or print how to run it manually.
pub fn run(project: &Path, role: &str) -> ExitCode {
    let valid_roles = ["client", "dedicated_server", "editor"];
    if !valid_roles.contains(&role) {
        eprintln!(
            "error: invalid role '{role}'. Expected one of: {}",
            valid_roles.join(", ")
        );
        return ExitCode::InvalidArgs;
    }

    let project_root = match project::resolve_project_root(project) {
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

    eprintln!(
        "Running project '{}' as role '{}' …",
        manifest.name, role
    );

    // Phase 0: only client role runs the demo binary; others print instructions.
    if role != "client" {
        eprintln!(
            "Role '{role}' is not yet implemented in Phase 0.\n\
             To run manually once supported:\n\
               cargo run -p {} -- --role {role}",
            manifest.name
        );
        return ExitCode::Success;
    }

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", &manifest.name]);

    match cmd.status() {
        Ok(status) if status.success() => ExitCode::Success,
        Ok(_) => ExitCode::InternalError,
        Err(err) => {
            eprintln!(
                "error: could not spawn cargo run (is this a workspace member?).\n\
                 Try manually: cargo run -p {}\n\
                 Details: {err}",
                manifest.name
            );
            ExitCode::InternalError
        }
    }
}
