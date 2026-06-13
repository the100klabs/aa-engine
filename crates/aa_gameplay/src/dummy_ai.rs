use bevy::prelude::*;

use aa_ability::{apply_gameplay_effect, AttributeSet, GameplayEffectAsset};
use aa_scene::Possesses;
use aa_tags::TagRegistry;

use crate::components::{ControlsPlayer, Pawn, PlayerController};
use crate::spawn::TrainingDummy;

/// Simple melee profile for the training dummy.
#[derive(Component, Debug)]
pub struct DummyCombat {
    pub range: f32,
    pub cooldown_secs: f32,
    pub timer: f32,
    pub damage_effect: String,
}

impl Default for DummyCombat {
    fn default() -> Self {
        Self {
            range: 3.0,
            cooldown_secs: 1.5,
            timer: 0.0,
            damage_effect: "effects/dummy_melee".into(),
        }
    }
}

/// Applies melee gameplay effects to the player when they enter dummy range.
#[allow(clippy::too_many_arguments)]
pub fn tick_dummy_combat(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    effects: Res<Assets<GameplayEffectAsset>>,
    tag_registry: Res<TagRegistry>,
    mut dummies: Query<(&Transform, &mut DummyCombat), With<TrainingDummy>>,
    controllers: Query<(&ControlsPlayer, &Possesses), With<PlayerController>>,
    pawns: Query<&Transform, With<Pawn>>,
    mut tags: Query<&mut aa_tags::GameplayTags>,
    mut attributes: Query<&mut AttributeSet>,
    mut active_effects: Query<&mut aa_ability::ActiveEffects>,
    mut cue_writer: MessageWriter<aa_ability::GameplayCueEvent>,
    mut damage_writer: MessageWriter<aa_ability::DamageAppliedEvent>,
) {
    let dt = time.delta_secs();

    let player_target = controllers.iter().next().and_then(|(controls, possesses)| {
        let pawn_ok = pawns.get(possesses.0).ok()?;
        let state_ok = attributes.get(controls.0).ok()?;
        if state_ok.get("Health").unwrap_or(1.0) <= 0.0 {
            return None;
        }
        Some((controls.0, pawn_ok.translation))
    });

    let Some((player_state, player_pos)) = player_target else {
        return;
    };

    for (dummy_transform, mut combat) in &mut dummies {
        combat.timer = (combat.timer - dt).max(0.0);
        let distance = dummy_transform.translation.distance(player_pos);
        if distance > combat.range || combat.timer > 0.0 {
            continue;
        }

        let path = format!("{}.ron", combat.damage_effect);
        let handle: Handle<GameplayEffectAsset> = asset_server.load(&path);
        if let Some(effect) = effects.get(&handle) {
            apply_gameplay_effect(
                &mut commands,
                player_state,
                effect,
                handle,
                &tag_registry,
                &mut tags,
                &mut attributes,
                &mut active_effects,
                &mut cue_writer,
                &mut damage_writer,
                player_state,
            );
        }
        combat.timer = combat.cooldown_secs;
    }
}
