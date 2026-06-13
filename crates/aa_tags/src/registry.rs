use bevy::prelude::*;
use std::collections::HashMap;

/// Interned gameplay tag identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(pub u32);

/// Maps tag names to interned ids (built from `TagDictionary` at startup).
#[derive(Resource, Debug, Default)]
pub struct TagRegistry {
    name_to_id: HashMap<String, TagId>,
    id_to_name: Vec<String>,
}

impl TagRegistry {
    pub fn register(&mut self, name: impl Into<String>) -> TagId {
        let name = name.into();
        if let Some(&id) = self.name_to_id.get(&name) {
            return id;
        }
        let id = TagId(self.id_to_name.len() as u32);
        self.id_to_name.push(name.clone());
        self.name_to_id.insert(name, id);
        id
    }

    pub fn id(&self, name: &str) -> Option<TagId> {
        self.name_to_id.get(name).copied()
    }

    pub fn name(&self, id: TagId) -> Option<&str> {
        self.id_to_name.get(id.0 as usize).map(String::as_str)
    }

    /// Parent match: `Ability.Cooldown` matches `Ability.Cooldown.Fireball`.
    pub fn matches(&self, container_tag: TagId, query_tag: TagId) -> bool {
        let Some(container_name) = self.name(container_tag) else {
            return false;
        };
        let Some(query_name) = self.name(query_tag) else {
            return false;
        };
        container_name == query_name
            || container_name.starts_with(&format!("{query_name}."))
            || query_name.starts_with(&format!("{container_name}."))
    }
}
