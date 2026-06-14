use aa_core::AaSchedule;
use bevy::prelude::*;

use crate::components::GameMode;
use crate::death::{detect_death, tick_respawn};
use crate::dummy_ai::tick_dummy_combat;
use crate::init::{finish_player_init, init_local_player};
use crate::spawn::{finish_dummy_init, spawn_training_dummy};

pub struct AaGameplayPlugin;

impl Plugin for AaGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameMode>().add_systems(
            Update,
            (
                init_local_player,
                finish_player_init,
                spawn_training_dummy,
                finish_dummy_init,
                detect_death,
                tick_respawn,
                tick_dummy_combat,
            )
                .chain()
                .after(AaSchedule::InitState),
        );
    }
}
