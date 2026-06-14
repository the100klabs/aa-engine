use aa_core::AaSchedule;
use bevy::prelude::*;

use crate::activate::{execute_ability_impls, tick_cooldowns, try_activate_abilities};
use crate::events::{AbilityConfirmedEvent, ApplyEffectRequest};
use crate::pending_effects::apply_pending_effects;
use crate::apply::tick_active_effects;
use crate::assets::{
    AttributeSetAsset, GameplayAbilityAsset, GameplayEffectAsset,
};
use crate::events::{AbilityActivatedEvent, DamageAppliedEvent, GameplayCueEvent};
use crate::loader::{
    AttributeSetAssetLoader, GameplayAbilityAssetLoader, GameplayEffectAssetLoader,
};
use crate::AbilityImplRegistry;

pub struct AaAbilityPlugin;

impl Plugin for AaAbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AttributeSetAsset>()
            .init_asset::<GameplayEffectAsset>()
            .init_asset::<GameplayAbilityAsset>()
            .init_asset_loader::<AttributeSetAssetLoader>()
            .init_asset_loader::<GameplayEffectAssetLoader>()
            .init_asset_loader::<GameplayAbilityAssetLoader>()
            .init_resource::<AbilityImplRegistry>()
            .add_message::<AbilityActivatedEvent>()
            .add_message::<DamageAppliedEvent>()
            .add_message::<GameplayCueEvent>()
            .add_message::<AbilityConfirmedEvent>()
            .add_message::<ApplyEffectRequest>()
            .add_systems(
                FixedUpdate,
                (
                    (try_activate_abilities, execute_ability_impls)
                        .chain()
                        .in_set(AaSchedule::AbilityFixed),
                    (
                        apply_pending_effects,
                        tick_active_effects,
                        tick_cooldowns,
                    )
                        .chain()
                        .in_set(AaSchedule::Effects),
                ),
            );
    }
}