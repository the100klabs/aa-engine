use bevy::prelude::*;
use bevy::reflect::TypePath;
use std::any::TypeId;
use std::collections::HashMap;

/// Stub registry mapping reflected types to their short type paths.
///
/// Intended to grow into a full schema index for editor and agent tooling.
#[derive(Resource, Default, Debug)]
pub struct ReflectRegistry {
    registered: HashMap<TypeId, String>,
}

impl ReflectRegistry {
    /// Records a reflected type for later schema lookup.
    pub fn register_type<T: Reflect + TypePath + 'static>(&mut self) {
        self.registered
            .insert(TypeId::of::<T>(), T::short_type_path().to_string());
    }

    /// Returns the short type path for a previously registered type, if any.
    pub fn type_path<T: 'static>(&self) -> Option<&str> {
        self.registered
            .get(&TypeId::of::<T>())
            .map(String::as_str)
    }

    /// Number of types currently registered.
    pub fn len(&self) -> usize {
        self.registered.len()
    }

    /// Returns `true` when no types have been registered yet.
    pub fn is_empty(&self) -> bool {
        self.registered.is_empty()
    }
}
