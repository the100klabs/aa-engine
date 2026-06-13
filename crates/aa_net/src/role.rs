use bevy::prelude::*;

/// Network role for a running process (complements `aa_core::AppRole`).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetRole {
    #[default]
    Offline,
    Client,
    DedicatedServer,
    ListenServer,
}
