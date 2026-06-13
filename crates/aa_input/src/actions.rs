use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Semantic action identifier (data-driven from RON).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InputActionId(pub String);

/// Value produced by an input action.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputActionValue {
    Digital(bool),
    Axis1D(f32),
    Axis2D(Vec2),
}

/// Fired when a mapped action changes state (PreUpdate / Input set).
#[derive(Message, Debug, Clone)]
pub struct InputActionEvent {
    pub action: InputActionId,
    pub value: InputActionValue,
}
