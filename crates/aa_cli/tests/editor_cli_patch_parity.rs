//! AS-05 — Rust `aa scene patch` dry-run matches bootstrap patch semantics.

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn run_aa_cli_patch(root: &PathBuf, scene: &str, patch: &str) -> Value {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "aa_cli",
            "--",
            "scene",
            "patch",
            "--scene",
            scene,
            "--patch",
            patch,
            "--dry-run",
            "--json",
        ])
        .current_dir(root)
        .output()
        .expect("spawn aa_cli scene patch");

    assert!(
        output.status.success(),
        "aa_cli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("aa_cli json")
}

fn run_bootstrap_patch(root: &PathBuf, scene: &str, patch: &str) -> Value {
    let output = Command::new("python3")
        .args([
            "docs/specs/tools/aa_bootstrap.py",
            "scene",
            "patch",
            "--scene",
            scene,
            "--patch",
            patch,
            "--dry-run",
            "--json",
        ])
        .current_dir(root)
        .output()
        .expect("spawn bootstrap scene patch");

    assert!(
        output.status.success(),
        "bootstrap failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("bootstrap json")
}

#[test]
fn editor_cli_patch_parity() {
    let root = repo_root();
    let scene = "examples/open_world_studio/assets/sectors/sector_0_0.ron";
    let patch = "docs/specs/fixtures/open_world_studio/add_campfire.scene_patch.json";

    let rust_json = run_aa_cli_patch(&root, scene, patch);
    let py_json = run_bootstrap_patch(&root, scene, patch);

    for key in ["ok", "dry_run", "patch_id", "target", "undo_token"] {
        assert_eq!(
            rust_json[key], py_json[key],
            "mismatch on field `{key}`"
        );
    }
    assert_eq!(rust_json["affected_files"], py_json["affected_files"]);
    assert_eq!(rust_json["affected_entities"], py_json["affected_entities"]);
    assert_eq!(rust_json["ops"], py_json["ops"]);
}
