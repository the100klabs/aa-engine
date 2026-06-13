use std::collections::HashSet;

use crate::{TagId, TagRegistry};

#[derive(Debug, Clone)]
pub enum TagQueryExpr {
    All(Vec<String>),
    Any(Vec<String>),
    None(Vec<String>),
}

/// Gameplay tag query (GAS `FGameplayTagQuery` equivalent).
#[derive(Debug, Clone)]
pub struct TagQuery {
    pub expr: TagQueryExpr,
}

impl TagQuery {
    pub fn has_all(tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            expr: TagQueryExpr::All(tags.into_iter().map(Into::into).collect()),
        }
    }

    pub fn has_any(tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            expr: TagQueryExpr::Any(tags.into_iter().map(Into::into).collect()),
        }
    }

    pub fn evaluate(&self, registry: &TagRegistry, container: &HashSet<TagId>) -> bool {
        match &self.expr {
            TagQueryExpr::All(names) => names.iter().all(|name| {
                registry
                    .id(name)
                    .is_some_and(|q| container.iter().any(|t| registry.matches(*t, q)))
            }),
            TagQueryExpr::Any(names) => names.iter().any(|name| {
                registry
                    .id(name)
                    .is_some_and(|q| container.iter().any(|t| registry.matches(*t, q)))
            }),
            TagQueryExpr::None(names) => !names.iter().any(|name| {
                registry
                    .id(name)
                    .is_some_and(|q| container.iter().any(|t| registry.matches(*t, q)))
            }),
        }
    }
}
