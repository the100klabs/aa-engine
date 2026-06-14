//! Spawns training dummy enemies for the combat slice.

use bevy::prelude::*;

use aa_ability::{ActiveEffects, AttributeSet, PendingAttributeSet};
use aa_experience::ExperienceReady;
use aa_scene::PendingInit;
use aa_tags::GameplayTags;

use crate::dummy_ai::DummyCombat;

#[derive(Component, Debug)]
pub struct TrainingDummy;

pub fn spawn_training_dummy(
    mut commands: Commands,
    mut ready: MessageReader<ExperienceReady>,
    asset_server: Res<AssetServer>,
    mut spawned: Local<bool>,
) {
    if *spawned {
        return;
    }

    for _ in ready.read() {
        let handle: Handle<aa_ability::AttributeSetAsset> =
            asset_server.load("attributes/hero_combat.ron");

        let mut combat = DummyCombat::default();
        if std::env::var("AA_PLAYTEST_SCENARIO").ok().as_deref() == Some("death_respawn") {
            combat.range = 10.0;
            combat.cooldown_secs = 0.5;
        }

        commands.spawn((
            TrainingDummy,
            combat,
            AttributeSet::default(),
            ActiveEffects::default(),
            GameplayTags::default(),
            PendingAttributeSet(handle),
            PendingInit,
            Transform::from_xyz(5.0, 1.0, 0.0),
            Name::new("TrainingDummy"),
        ));

        *spawned = true;
    }
}

pub fn finish_dummy_init(
    mut commands: Commands,
    attribute_assets: Res<Assets<aa_ability::AttributeSetAsset>>,
    mut dummies: Query<(Entity, &PendingAttributeSet, &mut AttributeSet), With<TrainingDummy>>,
) {
    for (entity, pending, mut attrs) in &mut dummies {
        let Some(asset) = attribute_assets.get(&pending.0) else {
            continue;
        };
        *attrs = aa_ability::attribute_set_from_asset(asset);
        commands.entity(entity).remove::<PendingInit>();
        commands.entity(entity).remove::<PendingAttributeSet>();
    }
}
