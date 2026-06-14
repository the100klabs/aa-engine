use bevy::prelude::*;

use aa_ability::{AttributeSet, ApplyEffectRequest};

use crate::components::{Pawn, PlayerState};
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
pub fn tick_dummy_combat(
    time: Res<Time>,
    mut dummies: Query<(&Transform, &mut DummyCombat), With<TrainingDummy>>,
    player_states: Query<(Entity, &AttributeSet), With<PlayerState>>,
    pawns: Query<&Transform, With<Pawn>>,
    mut effect_requests: MessageWriter<ApplyEffectRequest>,
) {
    let dt = time.delta_secs();

    let player_target = player_states.iter().next().and_then(|(state_entity, attrs)| {
        if attrs.get("Health").unwrap_or(1.0) <= 0.0 {
            return None;
        }
        let pos = pawns.iter().next()?.translation;
        Some((state_entity, pos))
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

        effect_requests.write(ApplyEffectRequest {
            target: player_state,
            effect_path: combat.damage_effect.clone(),
            source: player_state,
        });
        combat.timer = combat.cooldown_secs;
    }
}
