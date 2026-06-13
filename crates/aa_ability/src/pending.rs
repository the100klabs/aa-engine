use bevy::prelude::*;

use crate::AttributeSetAsset;

/// Tracks which attribute set asset to apply during init.
#[derive(Component, Debug, Clone)]
pub struct PendingAttributeSet(pub Handle<AttributeSetAsset>);
