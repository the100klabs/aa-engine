use std::path::Path;

use aa_world_stream::summarize_trace;

use crate::exit_codes::ExitCode;

/// Summarize a streaming profile trace JSON file.
pub fn summarize(trace_path: &Path, json: bool) -> ExitCode {
    let repo = find_workspace_root();
    let resolved = if trace_path.is_absolute() {
        trace_path.to_path_buf()
    } else {
        repo.join(trace_path)
    };

    let result = summarize_trace(&resolved, &repo);
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else if result.ok {
        eprintln!(
            "profile ok: {} (p95 load {:.1}ms, hitch {:.1}ms)",
            result.artifact,
            result.sector_streaming.load_latency.p95_ms,
            result.sector_streaming.crossing_hitch_ms
        );
    } else {
        eprintln!("profile budget status: {}", result.budget_status);
    }

    if result.ok {
        ExitCode::Success
    } else {
        ExitCode::ValidationFailed
    }
}

fn find_workspace_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
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
    std::path::PathBuf::from(".")
}
