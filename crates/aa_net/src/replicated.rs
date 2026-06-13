use bevy::prelude::*;

/// Marks an entity as participating in network replication.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Replicated;

/// Stable network identifier assigned by the server.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkId(pub u64);
