//! Asset registry, tag dictionary, and manifest loading for the AA Engine.

pub mod error;
pub mod manifest;
pub mod plugin;
pub mod registry;
pub mod tag_dictionary;
pub mod validation;

pub use error::AssetError;
pub use manifest::{load_asset_manifest, AssetManifest, AssetManifestEntry, ASSET_MANIFEST_PATH};
pub use plugin::AaAssetsPlugin;
pub use registry::AssetRegistry;
pub use tag_dictionary::{
    load_tag_dictionary, TagDictionary, TagDictionaryResource, TAG_DICTIONARY_PATH,
    TAG_DICTIONARY_SCHEMA_VERSION,
};
pub use validation::{tag_exists, validate_tag, validate_tags};
