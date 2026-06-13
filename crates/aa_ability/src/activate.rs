#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;

use aa_tags::{GameplayTags, TagQuery, TagRegistry};

use crate::ability::AbilityImplRegistry;
use crate::apply::apply_gameplay_effect;
use crate::assets::{GameplayAbilityAsset, GameplayEffectAsset};
use crate::components::AbilityRegistry;
use crate::events::{
    AbilityActivatedEvent, AbilityConfirmedEvent, DamageAppliedEvent, GameplayCueEvent,
};

/// Attempts to activate abilities when `AbilityActivatedEvent`s arrive.
pub fn try_activate_abilities(
    mut commands: Commands,
    mut activation_events: MessageReader<AbilityActivatedEvent>,
    mut ability_registry_query: Query<(Entity, &mut AbilityRegistry)>,
    abilities: Res<Assets<GameplayAbilityAsset>>,
    effects: Res<Assets<GameplayEffectAsset>>,
    _impl_registry: Res<AbilityImplRegistry>,
    tag_registry: Res<TagRegistry>,
    mut tags: Query<&mut GameplayTags>,
    mut attributes: Query<&mut crate::AttributeSet>,
    mut active_effects: Query<&mut crate::ActiveEffects>,
    mut cue_writer: MessageWriter<GameplayCueEvent>,
    mut damage_writer: MessageWriter<DamageAppliedEvent>,
    mut confirmed: MessageWriter<AbilityConfirmedEvent>,
) {
    for event in activation_events.read() {
        let caster = event.caster;
        let Ok((_, mut registry)) = ability_registry_query.get_mut(caster) else {
            continue;
        };

        let Some(granted) = registry
            .granted
            .iter()
            .find(|g| g.ability_id == event.ability_id)
        else {
            continue;
        };

        let Some(ability) = abilities.get(&granted.handle) else {
            continue;
        };

        if registry.cooldowns.get(&ability.id).copied().unwrap_or(0.0) > 0.0 {
            continue;
        }

        let mut activation_blocked = false;
        if let Ok(caster_tags) = tags.get(caster) {
            for blocked in &ability.activation_tags_blocked {
                let query = TagQuery::has_all([blocked.as_str()]);
                if caster_tags.evaluate(&tag_registry, &query) {
                    activation_blocked = true;
                    break;
                }
            }
        }
        if activation_blocked {
            continue;
        }

        if let Some(cost_path) = &ability.cost_effect
            && let Some((_, cost_effect)) = effects.iter().find(|(_, e)| e.id == *cost_path)
        {
            apply_gameplay_effect(
                &mut commands,
                caster,
                cost_effect,
                Handle::default(),
                &tag_registry,
                &mut tags,
                &mut attributes,
                &mut active_effects,
                &mut cue_writer,
                &mut damage_writer,
                caster,
            );
        }

        if let Some(cue) = &ability.cue_on_activate {
            cue_writer.write(GameplayCueEvent {
                cue: cue.clone(),
                entity: caster,
            });
        }

        confirmed.write(AbilityConfirmedEvent {
            caster,
            ability_id: event.ability_id.clone(),
            impl_key: ability.r#impl.clone(),
        });

        registry.cooldowns.insert(ability.id.clone(), 0.5);
    }
}

/// Runs registered ability implementations for confirmed activations.
pub fn execute_ability_impls(world: &mut World) {
    let events: Vec<AbilityConfirmedEvent> = {
        let mut confirmed = world.resource_mut::<Messages<AbilityConfirmedEvent>>();
        confirmed.drain().collect()
    };

    for event in events {
        let Some(func) = world.resource::<AbilityImplRegistry>().get(&event.impl_key) else {
            continue;
        };
        func(world, event.caster, &event.ability_id);
    }
}

/// Decrements ability cooldown timers.
pub fn tick_cooldowns(time: Res<Time>, mut query: Query<&mut AbilityRegistry>) {
    let dt = time.delta_secs();
    for mut registry in &mut query {
        for timer in registry.cooldowns.values_mut() {
            *timer = (*timer - dt).max(0.0);
        }
    }
}
