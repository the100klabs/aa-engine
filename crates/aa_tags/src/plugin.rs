use aa_assets::TagDictionaryResource;
use bevy::prelude::*;

use crate::TagRegistry;

pub struct AaTagsPlugin;

impl Plugin for AaTagsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TagRegistry>()
            .add_systems(Startup, build_tag_registry);
    }
}

fn build_tag_registry(
    mut registry: ResMut<TagRegistry>,
    dictionary: Res<TagDictionaryResource>,
) {
    for tag in &dictionary.dictionary().tags {
        registry.register(tag);
    }
}
