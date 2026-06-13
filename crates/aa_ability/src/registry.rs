use bevy::prelude::*;

use crate::attribute::AttributeSet;
use crate::components::{AbilityRegistry, GrantedAbility};
use crate::assets::{AttributeSetAsset, GameplayAbilityAsset};

/// Grants an ability handle to a PlayerState entity.
pub fn grant_ability(
    registry: &mut AbilityRegistry,
    ability_id: impl Into<String>,
    handle: Handle<GameplayAbilityAsset>,
) {
    registry.granted.push(GrantedAbility {
        ability_id: ability_id.into(),
        handle,
    });
}

/// Builds an `AttributeSet` component from a loaded asset.
pub fn attribute_set_from_asset(asset: &AttributeSetAsset) -> AttributeSet {
    let mut set = AttributeSet::default();
    for attr in &asset.attributes {
        set.insert_attribute(&attr.name, attr.default, attr.min, attr.max);
    }
    set
}
