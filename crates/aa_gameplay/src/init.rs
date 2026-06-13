use bevy::prelude::*;

use aa_ability::{
    attribute_set_from_asset, grant_ability, AbilityRegistry, ActiveEffects, AttributeSet,
    GameplayAbilityAsset, PendingAttributeSet,
};
use aa_experience::{ExperienceAction, ExperienceReady};
use aa_input::{ActiveInputContexts, InputMappingContextAsset};
use aa_scene::{PendingInit, PossessedBy, Possesses};
use aa_tags::GameplayTags;

use crate::components::{ControlsPlayer, Pawn, PlayerController, PlayerState};

/// Spawns local player graph after experience loads: PlayerState → Controller → Pawn.
pub fn init_local_player(
    mut commands: Commands,
    mut ready: MessageReader<ExperienceReady>,
    experiences: Res<Assets<aa_experience::ExperienceDefinitionAsset>>,
    asset_server: Res<AssetServer>,
    mut contexts: ResMut<ActiveInputContexts>,
    mut spawned: Local<bool>,
) {
    if *spawned {
        return;
    }

    for event in ready.read() {
        let Some(experience) = experiences.get(&event.handle) else {
            continue;
        };

        let mut ability_registry = AbilityRegistry::default();
        let mut attribute_handle: Option<Handle<aa_ability::AttributeSetAsset>> = None;

        for action in &experience.actions {
            match action {
                ExperienceAction::GrantAbilitySet { abilities } => {
                    for ability_path in abilities {
                        let handle: Handle<GameplayAbilityAsset> =
                            asset_server.load(format!("{ability_path}.ron"));
                        grant_ability(&mut ability_registry, ability_path, handle);
                    }
                }
                ExperienceAction::AddInputContext { context } => {
                    let handle: Handle<InputMappingContextAsset> =
                        asset_server.load(format!("{context}.ron"));
                    contexts.push_context(handle);
                }
                ExperienceAction::LoadAttributeSet { path } => {
                    attribute_handle = Some(asset_server.load(format!("{path}.ron")));
                }
            }
        }

        let mut player_state_cmds = commands.spawn((
            PlayerState { player_id: 0 },
            ability_registry,
            AttributeSet::default(),
            ActiveEffects::default(),
            GameplayTags::default(),
            PendingInit,
            Name::new("PlayerState"),
        ));

        if let Some(handle) = attribute_handle {
            player_state_cmds.insert(PendingAttributeSet(handle));
        }

        let player_state = player_state_cmds.id();

        let pawn = commands
            .spawn((
                Pawn,
                PendingInit,
                Transform::from_xyz(0.0, 1.0, 0.0),
                Name::new("PlayerPawn"),
            ))
            .id();

        let controller = commands
            .spawn((
                PlayerController { player_id: 0 },
                ControlsPlayer(player_state),
                Possesses(pawn),
                Name::new("PlayerController"),
            ))
            .id();

        commands.entity(pawn).insert(PossessedBy::new(controller));

        info!(
            "initialized local player for experience `{}`",
            experience.id
        );
        *spawned = true;
    }
}

/// Applies loaded attribute set assets and clears `PendingInit`.
pub fn finish_player_init(
    mut commands: Commands,
    attribute_assets: Res<Assets<aa_ability::AttributeSetAsset>>,
    mut player_states: Query<
        (Entity, &PendingAttributeSet, &mut AttributeSet),
        With<PlayerState>,
    >,
    mut pawns: Query<Entity, (With<Pawn>, With<PendingInit>)>,
) {
    for (entity, pending, mut attrs) in &mut player_states {
        let Some(asset) = attribute_assets.get(&pending.0) else {
            continue;
        };
        *attrs = attribute_set_from_asset(asset);
        commands.entity(entity).remove::<PendingInit>();
        commands.entity(entity).remove::<PendingAttributeSet>();
    }

    for pawn in &mut pawns {
        commands.entity(pawn).remove::<PendingInit>();
    }
}
