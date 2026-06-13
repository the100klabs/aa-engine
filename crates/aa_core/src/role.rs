use bevy::prelude::*;

/// Application role — set once at boot via [`AaCorePlugin`].
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppRole {
    #[default]
    Client,
    DedicatedServer,
    ListenServer,
    Editor,
}

impl AppRole {
    pub fn is_server(&self) -> bool {
        matches!(self, Self::DedicatedServer | Self::ListenServer)
    }
}
