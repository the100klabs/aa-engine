pub mod config;
pub mod cvar;
pub mod engine;
pub mod error;
pub mod paths;
pub mod plugin;
pub mod role;
pub mod schedule;

pub use config::ConfigProvider;
pub use cvar::CvarRegistry;
pub use engine::{add_core_plugin, init_project};
pub use error::ConfigError;
pub use paths::{project_path, project_root, set_project_root, PROJECT_ROOT_ENV};
pub use plugin::AaCorePlugin;
pub use role::AppRole;
pub use schedule::AaSchedule;

/// Supported schema version for all Phase 0 RON/JSON assets.
pub const SCHEMA_VERSION: u32 = 1;
