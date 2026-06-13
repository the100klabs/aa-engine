use bevy::prelude::*;

use aa_tags::TagRegistry;

use crate::apply::apply_gameplay_effect;
use crate::assets::GameplayEffectAsset;
use crate::events::ApplyEffectRequest;

/// Applies queued instant effects (used by ability impls such as heal).
#[allow(clippy::too_many_arguments)]
pub fn apply_pending_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    effects: Res<Assets<GameplayEffectAsset>>,
    tag_registry: Res<TagRegistry>,
    mut requests: MessageReader<ApplyEffectRequest>,
    mut tags: Query<&mut aa_tags::GameplayTags>,
    mut attributes: Query<&mut crate::AttributeSet>,
    mut active_effects: Query<&mut crate::ActiveEffects>,
    mut cue_writer: MessageWriter<crate::events::GameplayCueEvent>,
    mut damage_writer: MessageWriter<crate::events::DamageAppliedEvent>,
    mut pending: Local<Vec<(ApplyEffectRequest, Handle<GameplayEffectAsset>)>>,
) {
    for request in requests.read() {
        let path = format!("{}.ron", request.effect_path);
        pending.push((request.clone(), asset_server.load(&path)));
    }

    let mut i = 0;
    while i < pending.len() {
        let (request, handle) = &pending[i];
        let Some(effect) = effects.get(handle) else {
            i += 1;
            continue;
        };

        apply_gameplay_effect(
            &mut commands,
            request.target,
            effect,
            handle.clone(),
            &tag_registry,
            &mut tags,
            &mut attributes,
            &mut active_effects,
            &mut cue_writer,
            &mut damage_writer,
            request.source,
        );
        pending.remove(i);
    }
}
