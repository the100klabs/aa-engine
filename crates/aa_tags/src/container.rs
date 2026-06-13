use bevy::prelude::*;
use std::collections::HashSet;

use crate::{TagId, TagQuery, TagRegistry};

/// Owned gameplay tags on an entity (ASC tag container equivalent).
#[derive(Component, Debug, Default, Clone)]
pub struct GameplayTags {
    tags: HashSet<TagId>,
}

impl GameplayTags {
    pub fn insert(&mut self, tag: TagId) {
        self.tags.insert(tag);
    }

    pub fn remove(&mut self, tag: TagId) {
        self.tags.remove(&tag);
    }

    pub fn contains(&self, tag: TagId) -> bool {
        self.tags.contains(&tag)
    }

    pub fn iter(&self) -> impl Iterator<Item = TagId> + '_ {
        self.tags.iter().copied()
    }

    pub fn evaluate(&self, registry: &TagRegistry, query: &TagQuery) -> bool {
        query.evaluate(registry, &self.tags)
    }
}
