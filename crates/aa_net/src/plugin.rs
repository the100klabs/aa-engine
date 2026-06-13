use bevy::prelude::*;

use crate::role::NetRole;

pub struct AaNetPlugin {
    pub role: NetRole,
}

impl Default for AaNetPlugin {
    fn default() -> Self {
        Self {
            role: NetRole::Offline,
        }
    }
}

impl Plugin for AaNetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.role)
            .add_systems(Last, net_stub_tick);
    }
}

/// Phase 2 placeholder — transport and replication land here.
fn net_stub_tick(role: Res<NetRole>) {
    let _ = role;
}
