mod container;
mod plugin;
mod query;
mod registry;

pub use container::GameplayTags;
pub use plugin::AaTagsPlugin;
pub use query::{TagQuery, TagQueryExpr};
pub use registry::{TagId, TagRegistry};
