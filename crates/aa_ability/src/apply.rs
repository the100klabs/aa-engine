#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;

use aa_tags::{GameplayTags, TagQuery, TagRegistry};

use crate::assets::{EffectDuration, GameplayEffectAsset};
use crate::attribute::AttributeSet;
use crate::components::{ActiveEffectInstance, ActiveEffects};
use crate::events::{DamageAppliedEvent, GameplayCueEvent};

/// Applies a gameplay effect to a target entity (all attribute changes go through here).
pub fn apply_gameplay_effect(
    commands: &mut Commands,
    target: Entity,
    effect: &GameplayEffectAsset,
    effect_handle: Handle<GameplayEffectAsset>,
    tag_registry: &TagRegistry,
    tags: &mut Query<&mut GameplayTags>,
    attributes: &mut Query<&mut AttributeSet>,
    active_effects: &mut Query<&mut ActiveEffects>,
    cue_writer: &mut MessageWriter<GameplayCueEvent>,
    damage_writer: &mut MessageWriter<DamageAppliedEvent>,
    source: Entity,
) {
    if let Ok(target_tags) = tags.get(target) {
        for blocked in &effect.application_tags_blocked {
            let query = TagQuery::has_all([blocked.as_str()]);
            if target_tags.evaluate(tag_registry, &query) {
                return;
            }
        }
        for required in &effect.application_tags_required {
            let query = TagQuery::has_all([required.as_str()]);
            if !target_tags.evaluate(tag_registry, &query) {
                return;
            }
        }
    }

    match &effect.duration {
        EffectDuration::Instant => {
            if let Ok(mut attrs) = attributes.get_mut(target) {
                for modifier in &effect.modifiers {
                    let before = attrs.get(&modifier.attribute).unwrap_or(0.0);
                    attrs.apply_modifier(&modifier.attribute, modifier.op, modifier.magnitude);
                    let after = attrs.get(&modifier.attribute).unwrap_or(0.0);
                    if modifier.attribute == "Health" && after < before {
                        damage_writer.write(DamageAppliedEvent {
                            source,
                            target,
                            amount: before - after,
                        });
                    }
                }
            }
        }
        EffectDuration::Infinite | EffectDuration::Periodic { .. } => {
            if let Ok(mut active) = active_effects.get_mut(target) {
                let (remaining, period) = match &effect.duration {
                    EffectDuration::Periodic { count, .. } => (*count, effect_period(effect)),
                    EffectDuration::Infinite => (u32::MAX, 0.0),
                    EffectDuration::Instant => (0, 0.0),
                };
                active.effects.push(ActiveEffectInstance {
                    effect_id: effect.id.clone(),
                    handle: effect_handle,
                    remaining_ticks: remaining,
                    period_timer: period,
                });
            }
        }
    }

    if let Ok(mut target_tags) = tags.get_mut(target) {
        for tag_name in &effect.granted_tags {
            if let Some(tag) = tag_registry.id(tag_name) {
                target_tags.insert(tag);
            }
        }
    }

    for cue in &effect.cues_on_apply {
        cue_writer.write(GameplayCueEvent {
            cue: cue.clone(),
            entity: target,
        });
    }

    let _ = commands;
}

fn effect_period(effect: &GameplayEffectAsset) -> f32 {
    match &effect.duration {
        EffectDuration::Periodic { seconds, .. } => *seconds,
        _ => 0.0,
    }
}

/// Ticks periodic effects and removes expired instances.
pub fn tick_active_effects(
    time: Res<Time>,
    tag_registry: Res<TagRegistry>,
    effects_assets: Res<Assets<GameplayEffectAsset>>,
    mut tags: Query<&mut GameplayTags>,
    mut attributes: Query<&mut AttributeSet>,
    mut active_effects: Query<(Entity, &mut ActiveEffects)>,
    mut cue_writer: MessageWriter<GameplayCueEvent>,
    mut damage_writer: MessageWriter<DamageAppliedEvent>,
) {
    let dt = time.delta_secs();
    for (entity, mut active) in &mut active_effects {
        let mut i = 0;
        while i < active.effects.len() {
            let instance = &mut active.effects[i];
            instance.period_timer -= dt;
            if instance.period_timer > 0.0 {
                i += 1;
                continue;
            }

            if let Some(effect) = effects_assets.get(&instance.handle) {
                if let Ok(mut attrs) = attributes.get_mut(entity) {
                    for modifier in &effect.modifiers {
                        let before = attrs.get(&modifier.attribute).unwrap_or(0.0);
                        attrs.apply_modifier(&modifier.attribute, modifier.op, modifier.magnitude);
                        let after = attrs.get(&modifier.attribute).unwrap_or(0.0);
                        if modifier.attribute == "Health" && after < before {
                            damage_writer.write(DamageAppliedEvent {
                                source: entity,
                                target: entity,
                                amount: before - after,
                            });
                        }
                    }
                }
                instance.period_timer = effect_period(effect);
            }

            if instance.remaining_ticks != u32::MAX {
                instance.remaining_ticks = instance.remaining_ticks.saturating_sub(1);
            }

            if instance.remaining_ticks == 0 {
                active.effects.remove(i);
            } else {
                i += 1;
            }
        }

        let _ = (&mut tags, &tag_registry, &mut cue_writer);
    }
}
