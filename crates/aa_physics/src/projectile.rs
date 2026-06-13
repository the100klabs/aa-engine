use bevy::prelude::*;

use aa_gameplay::TrainingDummy;

use crate::movement::CharacterMovement;

/// Projectile spawned by combat abilities.
#[derive(Component, Debug)]
pub struct Projectile {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub damage_effect: String,
    pub owner: Entity,
}

/// Reads Move input and writes wish direction on possessed pawns.
pub fn gather_movement_intent(
    mut input_events: MessageReader<aa_input::InputActionEvent>,
    controllers: Query<&aa_scene::Possesses, With<aa_gameplay::PlayerController>>,
    mut movement: Query<&mut CharacterMovement>,
) {
    let mut move_axis = Vec2::ZERO;
    for event in input_events.read() {
        if event.action.0 == "Move" {
            move_axis = aa_input::axis2d(event.value);
        }
    }

    for possesses in &controllers {
        let Ok(mut character) = movement.get_mut(possesses.0) else {
            continue;
        };
        character.wish_dir = Vec3::new(move_axis.x, 0.0, -move_axis.y).normalize_or_zero();
    }
}

#[allow(clippy::type_complexity)]
pub fn tick_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut transforms: ParamSet<(
        Query<(Entity, &mut Projectile, &mut Transform)>,
        Query<(Entity, &Transform), With<TrainingDummy>>,
    )>,
    mut effect_requests: MessageWriter<aa_ability::ApplyEffectRequest>,
) {
    let dt = time.delta_secs();
    let hit_radius = 1.5;

    let dummy_positions: Vec<(Entity, Vec3)> = transforms
        .p1()
        .iter()
        .map(|(entity, transform)| (entity, transform.translation))
        .collect();

    for (entity, mut projectile, mut transform) in transforms.p0().iter_mut() {
        transform.translation += projectile.velocity * dt;
        projectile.lifetime -= dt;

        let mut hit_target = None;
        for (dummy, position) in &dummy_positions {
            if transform.translation.distance(*position) < hit_radius {
                hit_target = Some(*dummy);
                break;
            }
        }

        if let Some(target) = hit_target {
            queue_hit_effects(&mut effect_requests, target, projectile.owner, &projectile.damage_effect);
            commands.entity(entity).despawn();
            continue;
        }

        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn queue_hit_effects(
    writer: &mut MessageWriter<aa_ability::ApplyEffectRequest>,
    target: Entity,
    source: Entity,
    primary_effect: &str,
) {
    for effect_path in [primary_effect, "effects/burning"] {
        writer.write(aa_ability::ApplyEffectRequest {
            target,
            effect_path: effect_path.into(),
            source,
        });
    }
}
