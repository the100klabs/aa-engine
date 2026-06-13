use bevy::{asset::io::Reader, asset::AssetLoader, asset::LoadContext, prelude::*, reflect::TypePath};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use aa_core::SCHEMA_VERSION;

#[derive(Debug, Error)]
pub enum InputAssetLoaderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("unsupported schema_version {0}")]
    UnsupportedSchemaVersion(u32),
}

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct InputActionsAsset {
    pub schema_version: u32,
    pub actions: Vec<InputActionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputActionDef {
    pub id: String,
    pub value_type: InputValueType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputValueType {
    Digital,
    Axis1D,
    Axis2D,
}

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "InputMappingContext")]
pub struct InputMappingContextAsset {
    pub schema_version: u32,
    pub id: String,
    pub priority: i32,
    pub mappings: Vec<InputMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMapping {
    pub action: String,
    pub bindings: Vec<InputBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputBinding {
    Wasd,
    MouseDelta,
    MouseLeft,
    KeyboardSpace,
    KeyboardQ,
    KeyboardR,
    KeyboardE,
    GamepadLeftStick,
    GamepadRightStick,
    GamepadSouth,
    GamepadRightTrigger,
    GamepadLeftShoulder,
}

#[derive(Default, TypePath)]
pub struct InputActionsLoader;

impl AssetLoader for InputActionsLoader {
    type Asset = InputActionsAsset;
    type Settings = ();
    type Error = InputAssetLoaderError;

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset: InputActionsAsset = ron::de::from_bytes(&bytes)?;
        if asset.schema_version != SCHEMA_VERSION {
            return Err(InputAssetLoaderError::UnsupportedSchemaVersion(
                asset.schema_version,
            ));
        }
        Ok(asset)
    }
}

#[derive(Default, TypePath)]
pub struct InputMappingContextLoader;

impl AssetLoader for InputMappingContextLoader {
    type Asset = InputMappingContextAsset;
    type Settings = ();
    type Error = InputAssetLoaderError;

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset: InputMappingContextAsset = ron::de::from_bytes(&bytes)?;
        if asset.schema_version != SCHEMA_VERSION {
            return Err(InputAssetLoaderError::UnsupportedSchemaVersion(
                asset.schema_version,
            ));
        }
        Ok(asset)
    }
}

/// Active mapping contexts sorted by priority (highest first).
#[derive(Resource, Debug, Default)]
pub struct ActiveInputContexts {
    pub contexts: Vec<Handle<InputMappingContextAsset>>,
}

impl ActiveInputContexts {
    pub fn push_context(&mut self, handle: Handle<InputMappingContextAsset>) {
        self.contexts.push(handle);
    }
}

/// Resolved action lookup built from active contexts.
#[derive(Resource, Debug, Default)]
pub struct InputActionRegistry;
