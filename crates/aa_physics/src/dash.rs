use bevy::prelude::*;

/// Short burst of velocity applied on top of character movement (dash ability).
#[derive(Component, Debug)]
pub struct DashBurst {
    pub velocity: Vec3,
    pub remaining_secs: f32,
}

/// Integrates active dash bursts on pawns.
pub fn apply_dash_bursts(
    time: Res<Time>,
    mut bursts: Query<(Entity, &mut DashBurst, &mut crate::movement::CharacterMovement)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut burst, mut movement) in &mut bursts {
        burst.remaining_secs -= dt;
        movement.velocity += burst.velocity;
        if burst.remaining_secs <= 0.0 {
            commands.entity(entity).remove::<DashBurst>();
        }
    }
}
