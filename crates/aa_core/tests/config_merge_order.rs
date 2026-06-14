use std::path::PathBuf;

use aa_core::ConfigProvider;

fn open_world_studio_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/open_world_studio")
        .canonicalize()
        .expect("open_world_studio example must exist")
}

#[test]
fn config_merge_order_applies_engine_base_then_project_layers() {
    let provider = ConfigProvider::load(&open_world_studio_root()).expect("config should load");

    let title: String = provider
        .get("app.window_title")
        .expect("project engine.toml should override engine_base app.window_title");
    assert_eq!(title, "AA Open World Studio");

    let shadow: String = provider
        .get("render.shadow_quality")
        .expect("project render settings should merge");
    assert_eq!(shadow, "high");

    let max_teams: i64 = provider
        .get("teams.max_teams")
        .expect("project game.toml should merge teams table");
    assert_eq!(max_teams, 2);

    let default_port: i64 = provider
        .get("net.default_port")
        .expect("engine_base net defaults should remain when not overridden");
    assert_eq!(default_port, 7777);
}

#[test]
fn cli_override_wins_over_file_layers() {
    let mut provider = ConfigProvider::load(&open_world_studio_root()).expect("config should load");
    provider
        .apply_cli_override("app.target_fps", "30")
        .expect("cli override should parse");

    let fps: i64 = provider
        .get("app.target_fps")
        .expect("cli override should be readable");
    assert_eq!(fps, 30);
}
