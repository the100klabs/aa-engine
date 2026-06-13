use bevy::prelude::*;
use std::collections::HashMap;

/// Registers Rust ability implementations keyed by RON `impl` string.
#[derive(Resource, Default, Clone)]
pub struct AbilityImplRegistry {
    impls: HashMap<String, AbilityImplFn>,
}

pub type AbilityImplFn = fn(&mut World, Entity, &str);

impl AbilityImplRegistry {
    pub fn register(&mut self, key: impl Into<String>, func: AbilityImplFn) {
        self.impls.insert(key.into(), func);
    }

    pub fn get(&self, key: &str) -> Option<AbilityImplFn> {
        self.impls.get(key).copied()
    }
}
