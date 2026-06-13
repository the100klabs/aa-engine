use bevy::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::config::ConfigProvider;
use crate::cvar::CvarRegistry;
use crate::role::AppRole;
use crate::schedule::AaSchedule;

/// Core AA engine plugin — register before all other `aa_*` plugins.
pub struct AaCorePlugin {
    pub role: AppRole,
}

impl Default for AaCorePlugin {
    fn default() -> Self {
        Self {
            role: AppRole::Client,
        }
    }
}

impl Plugin for AaCorePlugin {
    fn build(&self, app: &mut App) {
        init_tracing();

        let config = ConfigProvider::load_from_env().unwrap_or_else(|err| {
            warn!("config load failed ({err}); using engine_base defaults only");
            ConfigProvider::with_engine_defaults(crate::paths::project_root())
        });

        app.insert_resource(self.role)
            .insert_resource(config)
            .init_resource::<CvarRegistry>()
            .configure_sets(First, AaSchedule::FrameStart)
            .configure_sets(
                PreUpdate,
                (
                    AaSchedule::Input,
                    AaSchedule::NetReceive,
                    AaSchedule::AbilityInput,
                )
                    .chain(),
            )
            .configure_sets(
                FixedUpdate,
                (
                    AaSchedule::MovementIntent,
                    AaSchedule::AbilityFixed,
                    AaSchedule::Physics,
                    AaSchedule::Effects,
                    AaSchedule::Crowd,
                    AaSchedule::RootMotion,
                )
                    .chain(),
            )
            .configure_sets(
                Update,
                (
                    AaSchedule::InitState,
                    AaSchedule::Animation,
                    AaSchedule::GameplayCues,
                    AaSchedule::WorldStream,
                    AaSchedule::Camera,
                )
                    .chain(),
            )
            .configure_sets(
                PostUpdate,
                (
                    AaSchedule::WorldStreamApply,
                    AaSchedule::NetSend,
                    AaSchedule::Interpolation,
                )
                    .chain(),
            )
            .configure_sets(Last, AaSchedule::FrameEnd);
    }
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("AA_LOG")
                .or_else(|_| EnvFilter::try_new("info"))
                .expect("valid default filter"),
        )
        .try_init();
}
