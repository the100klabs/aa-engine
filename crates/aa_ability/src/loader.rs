use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    reflect::TypePath,
};
use thiserror::Error;

use crate::assets::{
    validate_schema_version, AttributeSetAsset, GameplayAbilityAsset, GameplayEffectAsset,
};

#[derive(Debug, Error)]
pub enum AbilityAssetLoaderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("unsupported schema_version {0}")]
    UnsupportedSchemaVersion(u32),
}

#[derive(Default, TypePath)]
pub struct AttributeSetAssetLoader;

impl AssetLoader for AttributeSetAssetLoader {
    type Asset = AttributeSetAsset;
    type Settings = ();
    type Error = AbilityAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset: AttributeSetAsset = ron::de::from_bytes(&bytes)?;
        validate_schema_version(asset.schema_version)
            .map_err(AbilityAssetLoaderError::UnsupportedSchemaVersion)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Default, TypePath)]
pub struct GameplayEffectAssetLoader;

impl AssetLoader for GameplayEffectAssetLoader {
    type Asset = GameplayEffectAsset;
    type Settings = ();
    type Error = AbilityAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset: GameplayEffectAsset = ron::de::from_bytes(&bytes)?;
        validate_schema_version(asset.schema_version)
            .map_err(AbilityAssetLoaderError::UnsupportedSchemaVersion)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Default, TypePath)]
pub struct GameplayAbilityAssetLoader;

impl AssetLoader for GameplayAbilityAssetLoader {
    type Asset = GameplayAbilityAsset;
    type Settings = ();
    type Error = AbilityAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset: GameplayAbilityAsset = ron::de::from_bytes(&bytes)?;
        validate_schema_version(asset.schema_version)
            .map_err(AbilityAssetLoaderError::UnsupportedSchemaVersion)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
