use std::path::PathBuf;

use aa_core::ConfigProvider;

fn open_world_studio_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/open_world_studio")
        .canonicalize()
        .expect("open_world_studio example must exist")
}

fn demo_game_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/demo_game")
        .canonicalize()
        .expect("demo_game example must exist")
}

/// REQ-GLOBAL-050: engine_base → engine → platform → game → user → CLI.
#[test]
fn req_global_050_layer_precedence_in_temp_project() {
    let temp = tempfile::tempdir().expect("temp project dir");
    let root = temp.path();

    std::fs::create_dir_all(root.join("config/platforms")).expect("config dirs");

    std::fs::write(
        root.join("config/engine.toml"),
        r#"
[app]
marker = "engine"
target_fps = 120

[render]
shadow_quality = "medium"
msaa = 2
"#,
    )
    .expect("engine.toml");

    let platform_name = std::env::consts::OS;
    std::fs::write(
        root.join(format!("config/platforms/{platform_name}.toml")),
        r#"
[app]
marker = "platform"

[render]
shadow_quality = "high"
"#,
    )
    .expect("platform.toml");

    std::fs::write(
        root.join("config/game.toml"),
        r#"
[app]
marker = "game"

[teams]
max_teams = 8
"#,
    )
    .expect("game.toml");

    std::fs::write(
        root.join("config/user.toml"),
        r#"
[app]
marker = "user"
target_fps = 90
"#,
    )
    .expect("user.toml");

    let mut provider = ConfigProvider::load(root).expect("config should load");

    let marker: String = provider.get("app.marker").expect("user layer wins over game");
    assert_eq!(marker, "user");

    let fps: i64 = provider.get("app.target_fps").expect("user overrides engine fps");
    assert_eq!(fps, 90);

    let shadow: String = provider
        .get("render.shadow_quality")
        .expect("platform overrides engine render key");
    assert_eq!(shadow, "high");

    let msaa: i64 = provider
        .get("render.msaa")
        .expect("engine render key survives when platform omits it");
    assert_eq!(msaa, 2);

    let max_teams: i64 = provider.get("teams.max_teams").expect("game layer merges");
    assert_eq!(max_teams, 8);

    let port: i64 = provider
        .get("net.default_port")
        .expect("engine_base default remains");
    assert_eq!(port, 7777);

    provider
        .apply_cli_override("app.marker", "cli")
        .expect("cli override parses");
    let cli_marker: String = provider.get("app.marker").expect("cli is highest layer");
    assert_eq!(cli_marker, "cli");

    provider
        .apply_cli_override("app.target_fps", "30")
        .expect("cli override parses");
    let cli_fps: i64 = provider.get("app.target_fps").expect("cli beats user fps");
    assert_eq!(cli_fps, 30);
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
fn demo_game_engine_overrides_engine_base_window_title() {
    let provider = ConfigProvider::load(&demo_game_root()).expect("config should load");

    let title: String = provider
        .get("app.window_title")
        .expect("demo_game engine.toml should override engine_base");
    assert_eq!(title, "AA Demo Game");

    let max_teams: i64 = provider
        .get("teams.max_teams")
        .expect("demo_game game.toml merges teams");
    assert_eq!(max_teams, 4);
}

#[test]
fn platform_layer_applies_when_present() {
    let provider = ConfigProvider::load(&demo_game_root()).expect("config should load");

    let vsync: bool = provider
        .get("app.vsync")
        .expect("platform layer should expose app.vsync");
    let expected = std::env::consts::OS != "linux";
    assert_eq!(
        vsync, expected,
        "platform layer should set vsync per OS (linux=false, others=true)"
    );
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

#[test]
fn cli_override_accepts_bool_and_float() {
    let mut provider = ConfigProvider::load(&open_world_studio_root()).expect("config should load");

    provider
        .apply_cli_override("teams.friendly_fire", "true")
        .expect("bool cli override");
    let friendly: bool = provider.get("teams.friendly_fire").expect("bool value");
    assert!(friendly);

    provider
        .apply_cli_override("render.resolution_scale", "0.85")
        .expect("float cli override");
    let scale: f64 = provider.get("render.resolution_scale").expect("float value");
    assert!((scale - 0.85).abs() < f64::EPSILON);
}
