use bevy::prelude::*;

use aa_ability::AttributeSet;
use aa_tags::{GameplayTags, TagRegistry};

use crate::components::{Pawn, PlayerController, PlayerState};
use crate::spawn::TrainingDummy;
use aa_scene::Possesses;

#[derive(Component, Debug)]
pub struct RespawnTimer {
    pub remaining: f32,
}

type CombatantQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static AttributeSet,
        &'static mut GameplayTags,
        Has<PlayerState>,
        Has<TrainingDummy>,
    ),
    Or<(With<PlayerState>, With<TrainingDummy>)>,
>;

type RespawnBodyQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        Has<Pawn>,
        Has<TrainingDummy>,
    ),
    Or<(With<Pawn>, With<TrainingDummy>)>,
>;

/// Marks dead entities and starts respawn countdown.
pub fn detect_death(
    mut commands: Commands,
    tag_registry: Res<TagRegistry>,
    game_mode: Res<crate::components::GameMode>,
    mut combatants: CombatantQuery<'_, '_>,
    controllers: Query<(&Possesses,), With<PlayerController>>,
) {
    let dead_tag = tag_registry.id("State.Dead");

    for (entity, attrs, mut tags, is_player, is_dummy) in &mut combatants {
        if is_dead(&tag_registry, &tags) {
            continue;
        }
        if attrs.get("Health").unwrap_or(1.0) > 0.0 {
            continue;
        }
        mark_dead(&mut tags, dead_tag);
        commands.entity(entity).insert(RespawnTimer {
            remaining: game_mode.respawn_delay_secs,
        });

        if is_player {
            for (possesses,) in &controllers {
                commands.entity(possesses.0).insert(Visibility::Hidden);
            }
        }
        if is_dummy {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
}

/// Respawns entities after timer expires.
#[allow(clippy::too_many_arguments)]
pub fn tick_respawn(
    mut commands: Commands,
    time: Res<Time>,
    tag_registry: Res<TagRegistry>,
    mut timers: Query<(Entity, &mut RespawnTimer, &mut AttributeSet, &mut GameplayTags)>,
    mut bodies: RespawnBodyQuery<'_, '_>,
    player_states: Query<Entity, With<PlayerState>>,
    dummy_entities: Query<Entity, With<TrainingDummy>>,
) {
    let dead_tag = tag_registry.id("State.Dead");
    let dt = time.delta_secs();

    for (entity, mut timer, mut attrs, mut tags) in &mut timers {
        timer.remaining -= dt;
        if timer.remaining > 0.0 {
            continue;
        }

        attrs.set_current("Health", 100.0);
        if let Some(dead) = dead_tag {
            tags.remove(dead);
        }
        commands.entity(entity).remove::<RespawnTimer>();

        if player_states.get(entity).is_ok() {
            for (pawn_entity, mut transform, is_pawn, _) in &mut bodies {
                if !is_pawn {
                    continue;
                }
                *transform = Transform::from_xyz(0.0, 1.0, 0.0);
                commands.entity(pawn_entity).insert(Visibility::Visible);
            }
        }

        if dummy_entities.get(entity).is_ok() {
            if let Ok((_, mut transform, _, _)) = bodies.get_mut(entity) {
                *transform = Transform::from_xyz(5.0, 1.0, 0.0);
            }
            commands.entity(entity).insert(Visibility::Visible);
        }
    }
}

fn is_dead(registry: &TagRegistry, tags: &GameplayTags) -> bool {
    tags.evaluate(registry, &aa_tags::TagQuery::has_all(["State.Dead"]))
}

fn mark_dead(tags: &mut GameplayTags, dead_tag: Option<aa_tags::TagId>) {
    if let Some(dead) = dead_tag {
        tags.insert(dead);
    }
}
