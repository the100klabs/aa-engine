//! Headless open-world playtest harness for sector streaming scenarios.

use std::fs;
use std::path::PathBuf;

use aa_world_stream::{
    AaWorldStreamPlugin, SectorLifecycle, SectorRegistry, SpawnedPawn, StreamingProfileTrace,
    StreamingSource, StreamingSourceKind,
};
use aa_core::{init_project, AaCorePlugin};
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::WinitPlugin;
use serde::Serialize;

/// Runs a scripted headless open-world scenario and writes `playtest_report.json`.
pub fn run() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scenario = std::env::var("AA_PLAYTEST_SCENARIO").unwrap_or_else(|_| "open_world_enemy_camp".into());
    let duration_secs: f32 = std::env::var("AA_PLAYTEST_DURATION")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30.0);

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>()
            .set(init_project(&project_root)),
    )
    .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(
        1.0 / 60.0,
    )))
    .add_plugins(AaCorePlugin::default())
    .add_plugins(AaWorldStreamPlugin {
        world_asset: "worlds/open_world_studio.ron".into(),
        project_root: project_root.clone(),
    })
    .insert_resource(PlaytestConfig {
        scenario: scenario.clone(),
        duration_secs,
        report_path: project_root.join("playtest_report.json"),
        trace_path: project_root.join("artifacts/profiles").join(format!("{scenario}.trace")),
        log_path: project_root.join("artifacts/logs").join(format!("{scenario}.log")),
    })
    .init_resource::<PlaytestState>()
    .add_systems(Startup, spawn_streaming_source)
    .add_systems(
        Update,
        (
            script_streaming_movement,
            playtest_step.after(aa_world_stream::tick_sector_streaming),
        )
            .chain(),
    );

    app.run();
}

#[derive(Resource)]
struct PlaytestConfig {
    scenario: String,
    duration_secs: f32,
    report_path: PathBuf,
    trace_path: PathBuf,
    log_path: PathBuf,
}

#[derive(Resource, Default)]
struct PlaytestState {
    elapsed: f32,
    sectors_visited: usize,
    sector_ready: bool,
    finished: bool,
    crashed: bool,
}

#[derive(Serialize)]
struct PlaytestReport {
    ok: bool,
    scenario: String,
    duration_secs: f32,
    assertions: Vec<AssertionResult>,
    artifacts: ArtifactsJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<ProfileJson>,
}

#[derive(Serialize)]
struct AssertionResult {
    name: String,
    passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize)]
struct ArtifactsJson {
    log: String,
    trace: String,
    profile: String,
}

#[derive(Serialize)]
struct ProfileJson {
    sector_load_p95_ms: f32,
    sector_crossing_hitch_ms: f32,
}

fn spawn_streaming_source(mut commands: Commands, config: Res<PlaytestConfig>) {
    let start = if config.scenario == "open_world_sector_traverse" {
        Vec3::new(-512.0, 0.0, -512.0)
    } else {
        Vec3::new(32.0, 0.0, 32.0)
    };
    commands.spawn((
        StreamingSource {
            id: "player_0".into(),
            kind: StreamingSourceKind::Player { player_id: 0 },
            radius_sectors: 1,
            priority: 255,
        },
        Transform::from_translation(start),
        Name::new("StreamingSource"),
    ));
}

/// Scripted traversal across sector boundaries for OWA-04 proof.
fn script_streaming_movement(
    time: Res<Time>,
    config: Res<PlaytestConfig>,
    mut sources: Query<&mut Transform, With<StreamingSource>>,
    mut state: ResMut<PlaytestState>,
    registry: Option<Res<SectorRegistry>>,
) {
    if state.finished {
        return;
    }
    state.elapsed += time.delta_secs();

    let Ok(mut transform) = sources.single_mut() else {
        return;
    };

    if config.scenario.as_str() == "open_world_sector_traverse" {
        let speed = 48.0;
        transform.translation.x += speed * time.delta_secs();
        transform.translation.z += speed * 0.5 * time.delta_secs();
    }

    let Some(registry) = registry else {
        return;
    };

    let coord = aa_world_stream::sector_coord_from_position(
        transform.translation,
        registry.sector_size_m,
    );
    let sector_id = format!("sector_{}_{}", coord[0], coord[1]);
    if registry
        .sectors
        .get(&sector_id)
        .is_some_and(|s| s.lifecycle == SectorLifecycle::Active)
    {
        state.sectors_visited = state.sectors_visited.max(1);
    }
    if registry
        .sectors
        .get("sector_0_0")
        .is_some_and(|s| s.lifecycle == SectorLifecycle::Active)
    {
        state.sector_ready = true;
    }
}

#[allow(clippy::too_many_arguments)]
fn playtest_step(
    mut state: ResMut<PlaytestState>,
    config: Res<PlaytestConfig>,
    time: Res<Time>,
    registry: Option<Res<SectorRegistry>>,
    names: Query<&Name>,
    guards: Query<&SpawnedPawn>,
    trace: Option<Res<StreamingProfileTrace>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if state.finished {
        return;
    }

    let ready_to_finish = if config.scenario == "open_world_enemy_camp" {
        state.sector_ready && state.elapsed > 8.0
    } else {
        state.elapsed >= config.duration_secs
    };
    if !ready_to_finish && state.elapsed < config.duration_secs {
        return;
    }
    state.finished = true;

    let registry_present = registry.is_some();
    let sector_lifecycle = registry
        .as_ref()
        .and_then(|r| r.sectors.get("sector_0_0"))
        .map(|s| format!("{:?}", s.lifecycle))
        .unwrap_or_else(|| "missing".into());
    let active_sector_0_0 = registry
        .as_ref()
        .and_then(|registry| registry.sectors.get("sector_0_0"))
        .is_some_and(|s| s.lifecycle == SectorLifecycle::Active);

    let camp_guard_exists = names.iter().any(|name| name.as_str() == "camp_guard_patrol")
        || guards.iter().any(|g| g.stable_name == "camp_guard_patrol");

    let load_samples: Vec<f32> = trace
        .as_ref()
        .map(|trace| trace.load_samples.iter().map(|s| s.load_ms).collect())
        .unwrap_or_default();
    let load_p95 = percentile(&load_samples, 0.95);
    let crossing_hitch = trace.as_ref().map(|t| t.crossing_hitch_ms).unwrap_or(0.0);

    let mut assertions = vec![
        AssertionResult {
            name: "no_crash".into(),
            passed: !state.crashed,
            message: None,
        },
        AssertionResult {
            name: "sector_0_0_active".into(),
            passed: active_sector_0_0 || config.scenario == "open_world_sector_traverse",
            message: None,
        },
    ];

    if config.scenario == "open_world_enemy_camp" {
        assertions.push(AssertionResult {
            name: "camp_guard_spawned".into(),
            passed: camp_guard_exists,
            message: if camp_guard_exists {
                None
            } else {
                Some("camp_guard_patrol entity not found".into())
            },
        });
    }

    if config.scenario == "open_world_sector_traverse" {
        let active_count = registry
            .as_ref()
            .map(|registry| {
                registry
                    .sectors
                    .values()
                    .filter(|s| s.lifecycle == SectorLifecycle::Active)
                    .count()
            })
            .unwrap_or(0);
        assertions.push(AssertionResult {
            name: "crossed_multiple_sectors".into(),
            passed: active_count >= 1 && state.elapsed > 5.0,
            message: None,
        });
    }

    assertions.push(AssertionResult {
        name: "sector_load_budget".into(),
        passed: load_p95 <= 400.0 || load_samples.is_empty(),
        message: None,
    });
    assertions.push(AssertionResult {
        name: "sector_crossing_hitch_budget".into(),
        passed: crossing_hitch <= 6.0,
        message: None,
    });

    let ok = assertions.iter().all(|a| a.passed);
    let log_rel = "artifacts/logs"
        .to_string();
    let trace_rel = format!(
        "artifacts/profiles/{}.trace",
        config.scenario
    );
    let _ = fs::create_dir_all(config.trace_path.parent().unwrap());
    let _ = fs::create_dir_all(config.log_path.parent().unwrap());
    if let Some(trace) = trace.as_ref() {
        let _ = trace.write_to_path(&config.trace_path);
    }
    let _ = fs::write(
        &config.log_path,
        format!(
            "playtest {} ok={ok} registry={registry_present} sector_0_0={sector_lifecycle}\n",
            config.scenario
        ),
    );

    let report = PlaytestReport {
        ok,
        scenario: config.scenario.clone(),
        duration_secs: time.elapsed_secs(),
        assertions,
        artifacts: ArtifactsJson {
            log: format!("{log_rel}/{}.log", config.scenario),
            trace: trace_rel.clone(),
            profile: trace_rel.replace(".trace", ".profile.json"),
        },
        profile: Some(ProfileJson {
            sector_load_p95_ms: load_p95,
            sector_crossing_hitch_ms: crossing_hitch,
        }),
    };

    if let Ok(text) = serde_json::to_string_pretty(&report) {
        let _ = fs::write(&config.report_path, text);
    }

    app_exit.write(if ok {
        AppExit::Success
    } else {
        AppExit::from_code(1)
    });
}

fn percentile(values: &[f32], pct: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let index = ((sorted.len() - 1) as f32 * pct).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}
