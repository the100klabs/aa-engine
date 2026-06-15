//! P0-06 headless proxy — live world inspect boots a headless runtime within 30 seconds.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn aa_binary(root: &PathBuf) -> PathBuf {
    root.join("target/debug/aa")
}

#[test]
fn headless_cold_boot_world_inspect_live_under_30s() {
    let root = repo_root();
    let build = Command::new("cargo")
        .args(["build", "-q", "-p", "aa_cli"])
        .current_dir(&root)
        .status()
        .expect("build aa_cli");
    assert!(build.success(), "failed to build aa_cli");

    let started = Instant::now();
    let output = Command::new(aa_binary(&root))
        .args([
            "world",
            "inspect",
            "--project",
            "examples/open_world_studio",
            "--world",
            "open_world_studio",
            "--live",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .expect("spawn world inspect --live");

    let elapsed = started.elapsed();
    assert!(
        output.status.success(),
        "world inspect --live failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"ok\": true") || stdout.contains("\"ok\":true"));

    assert!(
        elapsed <= Duration::from_secs(30),
        "headless runtime boot took {elapsed:?}, expected <= 30s"
    );
}
