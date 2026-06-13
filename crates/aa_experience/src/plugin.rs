use aa_core::AaSchedule;
use bevy::prelude::*;

use crate::assets::ExperienceDefinitionAsset;
use crate::loader::ExperienceDefinitionLoader;

/// Fired once the default experience asset is loaded.
#[derive(Message, Debug, Clone)]
pub struct ExperienceReady {
    pub handle: Handle<ExperienceDefinitionAsset>,
}

pub struct AaExperiencePlugin {
    pub default_experience: String,
}

impl Default for AaExperiencePlugin {
    fn default() -> Self {
        Self {
            default_experience: "experiences/demo".into(),
        }
    }
}

impl Plugin for AaExperiencePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ExperienceDefinitionAsset>()
            .init_asset_loader::<ExperienceDefinitionLoader>()
            .add_message::<ExperienceReady>()
            .add_systems(
                Update,
                load_default_experience
                    .in_set(AaSchedule::InitState)
                    .run_if(not(resource_exists::<ExperienceLoaded>)),
            );
        app.insert_resource(ExperienceConfig {
            path: self.default_experience.clone(),
        });
    }
}

#[derive(Resource, Debug)]
pub struct ExperienceConfig {
    pub path: String,
}

#[derive(Resource, Debug)]
struct ExperienceLoaded;

#[derive(Default)]
struct ExperienceLoadState {
    handle: Option<Handle<ExperienceDefinitionAsset>>,
    ready_sent: bool,
}

fn load_default_experience(
    mut commands: Commands,
    config: Res<ExperienceConfig>,
    asset_server: Res<AssetServer>,
    experiences: Res<Assets<ExperienceDefinitionAsset>>,
    mut ready: MessageWriter<ExperienceReady>,
    mut state: Local<ExperienceLoadState>,
) {
    if state.handle.is_none() {
        state.handle = Some(asset_server.load(format!("{}.ron", config.path)));
    }

    let Some(handle) = state.handle.as_ref() else {
        return;
    };

    if experiences.get(handle).is_some() && !state.ready_sent {
        ready.write(ExperienceReady {
            handle: handle.clone(),
        });
        commands.insert_resource(ExperienceLoaded);
        state.ready_sent = true;
        info!("experience ready: {}", config.path);
    }
}
