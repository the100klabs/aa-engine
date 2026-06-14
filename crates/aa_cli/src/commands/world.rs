use std::path::Path;

use aa_world_stream::{cook_world, inspect_world, inspect_world_live};

use crate::exit_codes::ExitCode;
use crate::project::{self, ProjectError};

/// Inspect an authored world descriptor and its sector refs.
pub fn inspect(path: &Path, world: &str, live: bool, json: bool) -> ExitCode {
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

    let result = if live {
        inspect_world_live(&project_root, world)
    } else {
        inspect_world(&project_root, world, None)
    };

    if json {
        match serde_json::to_string_pretty(&result) {
            Ok(text) => println!("{text}"),
            Err(e) => {
                eprintln!("error: failed to serialize world inspect JSON: {e}");
                return ExitCode::InternalError;
            }
        }
    } else if result.ok {
        eprintln!(
            "world inspect ok: {} ({} sectors, {} layers, {}ms)",
            result.world,
            result.sector_count,
            result.layers.len(),
            result.duration_ms
        );
    } else {
        for diagnostic in &result.diagnostics {
            eprintln!("{}: {}", diagnostic.code, diagnostic.message);
        }
        for sector in &result.sectors {
            for missing in &sector.missing_refs {
                eprintln!("REF_MISSING: {missing}");
            }
        }
        eprintln!("world inspect failed: {}", result.world);
    }

    if result.ok {
        ExitCode::Success
    } else {
        ExitCode::ValidationFailed
    }
}

/// Cook and optionally verify deterministic sector artifacts.
pub fn cook(path: &Path, world: &str, verify: bool, json: bool) -> ExitCode {
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

    let result = cook_world(&project_root, world, verify);

    if json {
        match serde_json::to_string_pretty(&result) {
            Ok(text) => println!("{text}"),
            Err(e) => {
                eprintln!("error: failed to serialize world cook JSON: {e}");
                return ExitCode::InternalError;
            }
        }
    } else if result.ok {
        eprintln!(
            "world cook ok: {} ({} sectors, verified={}, {}ms)",
            result.world, result.sector_count, result.verified, result.duration_ms
        );
    } else {
        for diagnostic in &result.diagnostics {
            eprintln!("{}: {}", diagnostic.code, diagnostic.message);
        }
        eprintln!("world cook failed: {}", result.world);
    }

    if result.ok {
        ExitCode::Success
    } else {
        ExitCode::ValidationFailed
    }
}

// Back-compat entry for main.rs inspect subcommand.
#[allow(dead_code)]
pub fn run(path: &Path, world: &str, live: bool, json: bool) -> ExitCode {
    inspect(path, world, live, json)
}
