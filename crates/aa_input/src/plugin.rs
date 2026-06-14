use aa_core::AaSchedule;
use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::assets::{
    ActiveInputContexts, InputActionRegistry, InputActionsAsset, InputActionsLoader,
    InputMappingContextAsset, InputMappingContextLoader,
};
use crate::gather::gather_input;

pub struct AaInputPlugin;

impl Plugin for AaInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<InputActionsAsset>()
            .init_asset::<InputMappingContextAsset>()
            .init_asset_loader::<InputActionsLoader>()
            .init_asset_loader::<InputMappingContextLoader>()
            .init_resource::<ActiveInputContexts>()
            .init_resource::<InputActionRegistry>()
            .add_message::<crate::actions::InputActionEvent>()
            .add_systems(
                PreUpdate,
                gather_input
                    .in_set(AaSchedule::Input)
                    .after(InputSystems),
            );
    }
}
