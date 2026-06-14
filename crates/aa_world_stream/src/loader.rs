use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    reflect::TypePath,
};
use thiserror::Error;

use crate::assets::{
    SectorDescriptorAsset, SectorDescriptorAssetData, WorldDescriptorAsset, WorldDescriptorAssetData,
};

#[derive(Debug, Error)]
pub enum WorldStreamLoaderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

#[derive(Default, TypePath)]
pub struct WorldDescriptorAssetLoader;

impl AssetLoader for WorldDescriptorAssetLoader {
    type Asset = WorldDescriptorAsset;
    type Settings = ();
    type Error = WorldStreamLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        if let Ok(data) = ron::de::from_bytes::<WorldDescriptorAssetData>(&bytes) {
            return Ok(WorldDescriptorAsset::from(data));
        }
        let data: WorldDescriptorAssetData = ron::de::from_bytes(&bytes)?;
        Ok(WorldDescriptorAsset::from(data))
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Default, TypePath)]
pub struct SectorDescriptorAssetLoader;

impl AssetLoader for SectorDescriptorAssetLoader {
    type Asset = SectorDescriptorAsset;
    type Settings = ();
    type Error = WorldStreamLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        if let Ok(data) = ron::de::from_bytes::<SectorDescriptorAssetData>(&bytes) {
            return Ok(SectorDescriptorAsset::from(data));
        }
        let data: SectorDescriptorAssetData = ron::de::from_bytes(&bytes)?;
        Ok(SectorDescriptorAsset::from(data))
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
