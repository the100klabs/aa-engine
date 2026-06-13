use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    reflect::TypePath,
};
use thiserror::Error;

use crate::assets::{validate_schema_version, ExperienceDefinitionAsset};

#[derive(Debug, Error)]
pub enum ExperienceAssetLoaderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("unsupported schema_version {0}")]
    UnsupportedSchemaVersion(u32),
}

#[derive(Default, TypePath)]
pub struct ExperienceDefinitionLoader;

impl AssetLoader for ExperienceDefinitionLoader {
    type Asset = ExperienceDefinitionAsset;
    type Settings = ();
    type Error = ExperienceAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset: ExperienceDefinitionAsset = ron::de::from_bytes(&bytes)?;
        validate_schema_version(asset.schema_version)
            .map_err(ExperienceAssetLoaderError::UnsupportedSchemaVersion)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
