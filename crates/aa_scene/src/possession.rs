use bevy::prelude::*;

/// Controller → pawn possession link (source of truth).
#[derive(Component, Debug)]
#[relationship(relationship_target = PossessedBy)]
pub struct Possesses(#[relationship] pub Entity);

/// Pawn side of possession (one controller per pawn).
#[derive(Component, Debug)]
#[relationship_target(relationship = Possesses)]
pub struct PossessedBy(#[relationship] Entity);

impl PossessedBy {
    pub fn new(controller: Entity) -> Self {
        Self(controller)
    }
}
