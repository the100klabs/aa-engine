use aa_ability::AbilityActivatedEvent;
use aa_gameplay::{ControlsPlayer, PlayerController};
use aa_input::{InputActionEvent, InputActionValue};
use aa_physics::{DashBurst, Projectile};
use bevy::prelude::*;

use super::PawnOrigin;

/// Maps semantic input actions to data-driven ability ids.
const ABILITY_BINDINGS: &[(&str, &str)] = &[
    ("Fire", "abilities/fireball"),
    ("Ability1", "abilities/dash"),
    ("Ability2", "abilities/rifle_shot"),
    ("Ability3", "abilities/heal_burst"),
];

pub fn route_ability_input(
    mut input_events: MessageReader<InputActionEvent>,
    controllers: Query<&ControlsPlayer, With<PlayerController>>,
    mut ability_events: MessageWriter<AbilityActivatedEvent>,
) {
    for event in input_events.read() {
        if !matches!(event.value, InputActionValue::Digital(true)) {
            continue;
        }
        let Some((_, ability_id)) = ABILITY_BINDINGS
            .iter()
            .find(|(action, _)| *action == event.action.0)
        else {
            continue;
        };

        for controls in &controllers {
            ability_events.write(AbilityActivatedEvent {
                caster: controls.0,
                ability_id: (*ability_id).into(),
            });
        }
    }
}

pub fn register_ability_impls(mut registry: ResMut<aa_ability::AbilityImplRegistry>) {
    registry.register("fireball", fireball_impl);
    registry.register("dash", dash_impl);
    registry.register("rifle_shot", rifle_shot_impl);
    registry.register("heal_burst", heal_burst_impl);
}

fn fireball_impl(world: &mut World, caster: Entity, _ability_id: &str) {
    if apply_fireball_hitscan(world, caster) {
        return;
    }
    spawn_projectile(world, caster, 20.0, 3.0, "effects/fireball_hit", 0.3);
}

/// Applies fireball damage when a training dummy is inside the aim cone.
fn apply_fireball_hitscan(world: &mut World, caster: Entity) -> bool {
    let Some(origin) = world.get_resource::<PawnOrigin>() else {
        return false;
    };
    let forward = origin.forward.normalize();
    let start = origin.translation + Vec3::Y * 0.5;
    const CONE_DOT: f32 = 0.86;
    const MAX_RANGE: f32 = 40.0;

    let mut best: Option<(Entity, f32)> = None;
    let mut dummies = world.query_filtered::<(Entity, &Transform), With<aa_gameplay::TrainingDummy>>();
    for (entity, transform) in dummies.iter(world) {
        let to_target = transform.translation - start;
        let distance = to_target.length();
        if !(0.1..=MAX_RANGE).contains(&distance) {
            continue;
        }
        let dir = to_target / distance;
        let dot = forward.dot(dir);
        if dot < CONE_DOT {
            continue;
        }
        if best.is_none_or(|(_, best_dist)| distance < best_dist) {
            best = Some((entity, distance));
        }
    }

    let Some((target, _)) = best else {
        return false;
    };

    world
        .resource_mut::<Messages<aa_ability::ApplyEffectRequest>>()
        .write(aa_ability::ApplyEffectRequest {
            target,
            effect_path: "effects/fireball_hit".into(),
            source: caster,
        });
    true
}

fn rifle_shot_impl(world: &mut World, caster: Entity, _ability_id: &str) {
    spawn_projectile(world, caster, 35.0, 2.0, "effects/rifle_hit", 0.2);
}

fn dash_impl(world: &mut World, _caster: Entity, _ability_id: &str) {
    let Some(pawn) = world.get_resource::<PawnOrigin>().and_then(|p| p.pawn_entity) else {
        return;
    };
    let forward = world
        .get_resource::<PawnOrigin>()
        .map(|p| p.forward)
        .unwrap_or(Vec3::NEG_Z);
    world.entity_mut(pawn).insert(DashBurst {
        velocity: forward * 18.0,
        remaining_secs: 0.2,
    });
}

fn heal_burst_impl(world: &mut World, caster: Entity, _ability_id: &str) {
    world
        .resource_mut::<Messages<aa_ability::ApplyEffectRequest>>()
        .write(aa_ability::ApplyEffectRequest {
            target: caster,
            effect_path: "effects/heal_burst".into(),
            source: caster,
        });
}

fn spawn_projectile(
    world: &mut World,
    caster: Entity,
    speed: f32,
    lifetime: f32,
    damage_effect: &str,
    scale: f32,
) {
    let (origin, forward) = world
        .get_resource::<PawnOrigin>()
        .map(|p| (p.translation + Vec3::Y * 0.5, p.forward))
        .unwrap_or((Vec3::new(0.0, 1.5, 0.0), Vec3::NEG_Z));

    world.spawn((
        Projectile {
            velocity: forward * speed,
            lifetime,
            damage_effect: damage_effect.into(),
            owner: caster,
        },
        Transform::from_translation(origin).with_scale(Vec3::splat(scale)),
        Name::new("Projectile"),
    ));
}
