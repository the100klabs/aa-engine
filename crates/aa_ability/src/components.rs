use bevy::prelude::*;
use std::collections::HashMap;

/// Granted abilities and cooldown tracking on PlayerState (not Pawn).
#[derive(Component, Debug, Default, Clone)]
pub struct AbilityRegistry {
    pub granted: Vec<GrantedAbility>,
    pub cooldowns: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct GrantedAbility {
    pub ability_id: String,
    pub handle: Handle<crate::GameplayAbilityAsset>,
}

/// Active gameplay effects on an entity.
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveEffects {
    pub effects: Vec<ActiveEffectInstance>,
}

#[derive(Debug, Clone)]
pub struct ActiveEffectInstance {
    pub effect_id: String,
    pub handle: Handle<crate::GameplayEffectAsset>,
    pub remaining_ticks: u32,
    pub period_timer: f32,
}
