use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct AbilityActivatedEvent {
    pub caster: Entity,
    pub ability_id: String,
}

#[derive(Message, Debug, Clone)]
pub struct DamageAppliedEvent {
    pub source: Entity,
    pub target: Entity,
    pub amount: f32,
}

#[derive(Message, Debug, Clone)]
pub struct GameplayCueEvent {
    pub cue: String,
    pub entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct ApplyEffectRequest {
    pub target: Entity,
    pub effect_path: String,
    pub source: Entity,
}

/// Emitted after an ability passes validation (for game-specific side effects).
#[derive(Message, Debug, Clone)]
pub struct AbilityConfirmedEvent {
    pub caster: Entity,
    pub ability_id: String,
    pub impl_key: String,
}
