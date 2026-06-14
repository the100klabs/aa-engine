use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    reflect::TypePath,
};
use thiserror::Error;

use crate::prefab::{PrefabAsset, PrefabAssetData};
use crate::scene::{SceneAsset, SceneRon};
use aa_core::SCHEMA_VERSION;

#[derive(Debug, Error)]
pub enum RonAssetLoaderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("unsupported schema_version {0}")]
    UnsupportedSchemaVersion(u32),
}

#[derive(Default, TypePath)]
pub struct PrefabAssetLoader;

impl AssetLoader for PrefabAssetLoader {
    type Asset = PrefabAsset;
    type Settings = ();
    type Error = RonAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let data: PrefabAssetData = ron::de::from_bytes(&bytes)?;
        if data.schema_version != SCHEMA_VERSION {
            return Err(RonAssetLoaderError::UnsupportedSchemaVersion(
                data.schema_version,
            ));
        }
        Ok(data.into())
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Default, TypePath)]
pub struct SceneAssetLoader;

impl AssetLoader for SceneAssetLoader {
    type Asset = SceneAsset;
    type Settings = ();
    type Error = RonAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let SceneRon::Scene(data) = ron::de::from_bytes::<SceneRon>(&bytes)?;
        if data.schema_version != SCHEMA_VERSION {
            return Err(RonAssetLoaderError::UnsupportedSchemaVersion(
                data.schema_version,
            ));
        }
        Ok(data.into())
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
