use bevy::prelude::*;
use std::collections::HashSet;

/// Index of known asset paths under the project `assets/` root.
#[derive(Debug, Default, Resource)]
pub struct AssetRegistry {
    paths: HashSet<String>,
}

impl AssetRegistry {
    /// Registers a known asset path.
    pub fn register(&mut self, path: impl Into<String>) {
        self.paths.insert(path.into());
    }

    /// Returns `true` when the path is present in the registry.
    pub fn contains(&self, path: &str) -> bool {
        self.paths.contains(path)
    }

    /// Resolves a soft reference path, returning `None` when missing.
    pub fn resolve(&self, path: &str) -> Option<&str> {
        self.paths.get(path).map(String::as_str)
    }

    /// Iterates all registered asset paths.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.paths.iter().map(String::as_str)
    }

    /// Number of registered asset paths.
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Returns `true` when no paths have been registered.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}
