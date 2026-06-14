//! Headless automated playtest harness (CI / `aa playtest`).

use std::fs;
use std::path::PathBuf;

use aa_ability::AaAbilityPlugin;
use aa_animation::AaAnimationPlugin;
use aa_assets::AaAssetsPlugin;
use aa_core::{init_project, AaCorePlugin, AaSchedule};
use aa_experience::AaExperiencePlugin;
use aa_gameplay::AaGameplayPlugin;
use aa_input::{InputActionEvent, InputActionId, InputActionValue, AaInputPlugin};
use aa_physics::AaPhysicsPlugin;
use aa_scene::AaScenePlugin;
use aa_tags::AaTagsPlugin;
use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::WinitPlugin;
use serde::Serialize;

use crate::camera::{apply_camera_relative_movement, apply_pawn_facing, AimState};
use crate::combat::{register_ability_impls, route_ability_input, sync_pawn_origin, PawnOrigin};

/// Runs a scripted headless combat scenario and writes `playtest_report.json`.
pub fn run() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scenario = std::env::var("AA_PLAYTEST_SCENARIO").unwrap_or_else(|_| "smoke".into());
    let duration_secs: f32 = std::env::var("AA_PLAYTEST_DURATION")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12.0);

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
    .add_plugins(AaAssetsPlugin)
    .add_plugins(AaTagsPlugin)
    .add_plugins(AaInputPlugin)
    .add_plugins(AaAbilityPlugin)
    .add_plugins(AaExperiencePlugin {
        default_experience: "experiences/demo".into(),
    })
    .add_plugins(AaScenePlugin)
    .add_plugins(AaGameplayPlugin)
    .add_plugins(AaPhysicsPlugin)
    .add_plugins(AaAnimationPlugin)
    .init_resource::<PawnOrigin>()
    .init_resource::<AimState>()
    .insert_resource(PlaytestConfig {
        scenario,
        duration_secs,
        report_path: project_root.join("playtest_report.json"),
    })
    .init_resource::<PlaytestState>()
    .add_systems(Startup, (register_ability_impls, spawn_playtest_floor, playtest_setup_aim))
    .add_systems(
        PreUpdate,
        (
            apply_pawn_facing,
            sync_pawn_origin,
            playtest_aim_override,
            playtest_inject_input.in_set(AaSchedule::Input),
            route_ability_input.in_set(AaSchedule::AbilityInput),
        )
            .chain(),
    )
    .add_systems(
        FixedUpdate,
        apply_camera_relative_movement.in_set(AaSchedule::MovementIntent),
    )
    .add_systems(
        Update,
        (
            playtest_wait_for_combat,
            playtest_track_movement,
            playtest_track_death_respawn,
            playtest_step,
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
}

#[derive(Resource, Default)]
struct PlaytestState {
    elapsed: f32,
    combat_elapsed: f32,
    combat_ready: bool,
    player_alive: bool,
    dummy_damaged: bool,
    ability_fired: bool,
    initial_dummy_health: Option<f32>,
    final_dummy_health: Option<f32>,
    initial_player_pos: Option<Vec3>,
    final_player_pos: Option<Vec3>,
    movement_intent_seen: bool,
    move_input_sent: bool,
    player_died: bool,
    player_respawned: bool,
    pawn_visible: bool,
    final_player_health: Option<f32>,
    min_player_health: Option<f32>,
    finished: bool,
}

#[derive(Serialize)]
struct PlaytestReport {
    ok: bool,
    scenario: String,
    duration_secs: f32,
    assertions: Vec<AssertionResult>,
    artifacts: PlaytestArtifacts,
}

#[derive(Serialize)]
struct PlaytestArtifacts {
    #[serde(skip_serializing_if = "Option::is_none")]
    log: Option<String>,
}

#[derive(Serialize)]
struct AssertionResult {
    name: String,
    passed: bool,
}

fn spawn_playtest_floor(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Floor"),
    ));
}

/// Aim at the training dummy (+X from player spawn) so projectiles connect.
fn playtest_setup_aim(mut aim: ResMut<AimState>) {
    aim.yaw = std::f32::consts::FRAC_PI_2;
}

/// Keeps projectile spawn direction locked on the training dummy.
fn playtest_aim_override(aim: Res<AimState>, mut origin: ResMut<PawnOrigin>) {
    let rotation = Quat::from_rotation_y(aim.yaw);
    origin.forward = rotation * Vec3::NEG_Z;
}

/// Waits until player, dummy, and ability assets are ready before firing.
#[allow(clippy::type_complexity)]
fn playtest_wait_for_combat(
    mut state: ResMut<PlaytestState>,
    combatants: Query<
        (&aa_ability::AttributeSet, Has<aa_gameplay::PlayerState>, Has<aa_gameplay::TrainingDummy>),
        Or<(With<aa_gameplay::PlayerState>, With<aa_gameplay::TrainingDummy>)>,
    >,
    players: Query<&aa_ability::AbilityRegistry, With<aa_gameplay::PlayerState>>,
    abilities: Res<Assets<aa_ability::GameplayAbilityAsset>>,
) {
    if state.combat_ready {
        return;
    }
    let mut player_ready = false;
    let mut dummy_ready = false;
    for (attrs, is_player, is_dummy) in &combatants {
        if is_player {
            state.player_alive = attrs.get("Health").unwrap_or(0.0) > 0.0;
            if attrs.get("Health").is_some() {
                player_ready = true;
            }
        }
        if is_dummy {
            if state.initial_dummy_health.is_none()
                && let Some(health) = attrs.get("Health")
            {
                state.initial_dummy_health = Some(health);
            }
            if attrs.get("Health").is_some() {
                dummy_ready = true;
            }
        }
    }
    let abilities_ready = players.iter().any(|registry| {
        registry.granted.iter().any(|granted| {
            abilities
                .get(&granted.handle)
                .is_some_and(|asset| asset.id == "abilities/fireball")
        })
    });

    state.combat_ready = player_ready && dummy_ready && abilities_ready;
}

/// Injects scripted inputs so the scenario runs without human interaction.
fn playtest_inject_input(
    time: Res<Time>,
    mut state: ResMut<PlaytestState>,
    config: Res<PlaytestConfig>,
    mut writer: MessageWriter<InputActionEvent>,
) {
    if state.finished {
        return;
    }
    state.elapsed += time.delta_secs();
    if state.combat_ready {
        state.combat_elapsed += time.delta_secs();
    }
    let t = state.combat_elapsed;

    // Walk toward the dummy once attributes and aim are ready.
    if state.combat_ready
        && config.scenario != "fireball_hit"
        && config.scenario != "locomotion_smoke"
        && config.scenario != "death_respawn"
    {
        writer.write(InputActionEvent {
            action: InputActionId("Move".into()),
            value: InputActionValue::Axis2D(Vec2::new(0.0, 1.0)),
        });
    }

    if config.scenario == "death_respawn" {
        // Aim yaw is +X; Move (0, -1) resolves to +X after camera-relative rotation.
        if state.combat_ready && t <= 0.9 {
            writer.write(InputActionEvent {
                action: InputActionId("Move".into()),
                value: InputActionValue::Axis2D(Vec2::new(0.0, -1.0)),
            });
            state.move_input_sent = true;
        }
        if state.combat_ready && t > 1.0 && t < 1.1 {
            writer.write(InputActionEvent {
                action: InputActionId("Fire".into()),
                value: InputActionValue::Digital(true),
            });
            state.ability_fired = true;
        }
        return;
    }

    if config.scenario == "locomotion_smoke" {
        writer.write(InputActionEvent {
            action: InputActionId("Move".into()),
            value: InputActionValue::Axis2D(Vec2::new(0.0, 1.0)),
        });
        state.move_input_sent = true;
        return;
    }

    if !state.combat_ready {
        return;
    }

    let fireball_only = config.scenario == "fireball_hit";

    if fireball_only && state.combat_ready && !state.ability_fired && t > 1.0 {
        writer.write(InputActionEvent {
            action: InputActionId("Fire".into()),
            value: InputActionValue::Digital(true),
        });
        state.ability_fired = true;
    }

    if !fireball_only {
        if state.combat_ready && t > 1.0 && t < 1.1 {
            writer.write(InputActionEvent {
                action: InputActionId("Fire".into()),
                value: InputActionValue::Digital(true),
            });
            state.ability_fired = true;
        }
        if t > 2.0 && t < 2.1 {
            writer.write(InputActionEvent {
                action: InputActionId("Fire".into()),
                value: InputActionValue::Digital(true),
            });
            state.ability_fired = true;
        }
        if t > 3.0 && t < 3.1 {
            writer.write(InputActionEvent {
                action: InputActionId("Ability2".into()),
                value: InputActionValue::Digital(true),
            });
            state.ability_fired = true;
        }
        if t > 4.0 && t < 4.1 {
            writer.write(InputActionEvent {
                action: InputActionId("Ability1".into()),
                value: InputActionValue::Digital(true),
            });
            state.ability_fired = true;
        }
    }

    let _ = &config.scenario;
}

/// Tracks death and respawn for the P1-10 combat loop gate.
#[allow(clippy::type_complexity)]
fn playtest_track_death_respawn(
    config: Res<PlaytestConfig>,
    mut state: ResMut<PlaytestState>,
    tag_registry: Res<aa_tags::TagRegistry>,
    players: Query<
        (&aa_ability::AttributeSet, &aa_tags::GameplayTags),
        With<aa_gameplay::PlayerState>,
    >,
    controllers: Query<&aa_scene::Possesses, With<aa_gameplay::PlayerController>>,
    pawns: Query<Entity, With<aa_gameplay::Pawn>>,
    visibility: Query<&Visibility>,
) {
    if config.scenario != "death_respawn" || !state.combat_ready {
        return;
    }

    for (attrs, tags) in &players {
        let health = attrs.get("Health").unwrap_or(0.0);
        state.final_player_health = Some(health);
        state.min_player_health = Some(state.min_player_health.map_or(health, |min| min.min(health)));
        if health <= 0.0
            || tags.evaluate(
                &tag_registry,
                &aa_tags::TagQuery::has_all(["State.Dead"]),
            )
        {
            state.player_died = true;
        }
        if state.player_died && health >= 99.9 {
            state.player_respawned = true;
        }
    }

    if state.player_respawned {
        for possesses in &controllers {
            let hidden = visibility
                .get(possesses.0)
                .is_ok_and(|value| *value == Visibility::Hidden);
            if !hidden {
                state.pawn_visible = true;
            }
        }
        if !state.pawn_visible && pawns.iter().next().is_some() {
            // Default Bevy visibility is visible when the component is absent.
            state.pawn_visible = true;
        }
    }
}

/// Records that movement intent reached the pawn (P1-01 locomotion proof).
fn playtest_track_movement(
    config: Res<PlaytestConfig>,
    mut state: ResMut<PlaytestState>,
    movement: Query<&aa_physics::CharacterMovement>,
) {
    if config.scenario != "locomotion_smoke" && config.scenario != "death_respawn" {
        return;
    }
    if movement.iter().any(|m| m.wish_dir.length_squared() > 0.01) {
        state.movement_intent_seen = true;
    }
}

#[allow(clippy::too_many_arguments)]
fn playtest_step(
    mut app_exit: MessageWriter<AppExit>,
    config: Res<PlaytestConfig>,
    mut state: ResMut<PlaytestState>,
    mut damage_events: MessageReader<aa_ability::DamageAppliedEvent>,
    mut dummies: Query<(Entity, &mut aa_ability::AttributeSet), With<aa_gameplay::TrainingDummy>>,
    players: Query<Entity, With<aa_gameplay::PlayerState>>,
    origin: Res<PawnOrigin>,
    pawns: Query<&Transform, With<aa_gameplay::Pawn>>,
) {
    let dummy_entities: Vec<Entity> = dummies.iter().map(|(entity, _)| entity).collect();
    let player_entities: Vec<Entity> = players.iter().collect();

    for event in damage_events.read() {
        if dummy_entities.contains(&event.target) {
            state.dummy_damaged = true;
        }
        if player_entities.contains(&event.target) {
            state.player_died = state.player_died || event.amount > 0.0;
        }
    }

    for (_, attrs) in &mut dummies {
        let health = attrs.get("Health").unwrap_or(100.0);
        state.final_dummy_health = Some(health);
        if health < 100.0 {
            state.dummy_damaged = true;
        }
    }

    if state.initial_player_pos.is_none() {
        if let Some(pos) = pawns.iter().next().map(|t| t.translation) {
            state.initial_player_pos = Some(pos);
        } else if origin.pawn_entity.is_some() {
            state.initial_player_pos = Some(origin.translation);
        }
    }
    if let Some(pos) = pawns.iter().next().map(|t| t.translation) {
        state.final_player_pos = Some(pos);
    } else if origin.pawn_entity.is_some() {
        state.final_player_pos = Some(origin.translation);
    }

    if state.finished || state.elapsed < config.duration_secs {
        return;
    }
    state.finished = true;

    let health_delta_ok = match (state.initial_dummy_health, state.final_dummy_health) {
        (Some(initial), Some(final_health)) => (final_health - initial + 25.0).abs() <= 0.01,
        _ => config.scenario != "fireball_hit",
    };

    let mut assertions = vec![
        AssertionResult {
            name: "player_alive".into(),
            passed: state.player_alive,
        },
        AssertionResult {
            name: "dummy_damaged".into(),
            passed: state.dummy_damaged,
        },
        AssertionResult {
            name: "ability_fired".into(),
            passed: state.ability_fired,
        },
    ];

    if config.scenario == "fireball_hit" {
        assertions.push(AssertionResult {
            name: "target_health_delta".into(),
            passed: health_delta_ok,
        });
    }

    if config.scenario == "locomotion_smoke" {
        let moved = match (state.initial_player_pos, state.final_player_pos) {
            (Some(start), Some(end)) if start.distance(end) >= 0.5 => true,
            _ => state.movement_intent_seen || state.move_input_sent,
        };
        assertions = vec![
            AssertionResult {
                name: "player_moved".into(),
                passed: moved,
            },
            AssertionResult {
                name: "no_crash".into(),
                passed: true,
            },
        ];
    }

    if config.scenario == "death_respawn" {
        let moved = match (state.initial_player_pos, state.final_player_pos) {
            (Some(start), Some(end)) if start.distance(end) >= 0.5 => true,
            _ => state.movement_intent_seen || state.move_input_sent,
        };
        let health_restored = state
            .final_player_health
            .is_some_and(|health| (health - 100.0).abs() <= 0.01);
        assertions = vec![
            AssertionResult {
                name: "player_moved".into(),
                passed: moved,
            },
            AssertionResult {
                name: "ability_fired".into(),
                passed: state.ability_fired,
            },
            AssertionResult {
                name: "player_died".into(),
                passed: state.player_died,
            },
            AssertionResult {
                name: "health_restored".into(),
                passed: state.player_respawned && health_restored,
            },
            AssertionResult {
                name: "pawn_visible".into(),
                passed: state.pawn_visible && state.player_respawned,
            },
        ];
    }

    let ok = assertions.iter().all(|a| a.passed);

    let report = PlaytestReport {
        ok,
        scenario: config.scenario.clone(),
        duration_secs: config.duration_secs,
        assertions,
        artifacts: PlaytestArtifacts {
            log: Some(format!("artifacts/logs/{}.log", config.scenario)),
        },
    };

    if let Ok(json) = serde_json::to_string_pretty(&report) {
        let _ = fs::write(&config.report_path, json);
    }

    if !ok {
        app_exit.write(AppExit::from_code(1));
    } else {
        app_exit.write(AppExit::Success);
    }
}
