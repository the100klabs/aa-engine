use bevy::prelude::*;

use crate::actions::{InputActionEvent, InputActionId, InputActionValue};
use crate::assets::{ActiveInputContexts, InputBinding, InputMappingContextAsset};

/// Reads keyboard/mouse and emits semantic `InputActionEvent`s.
pub fn gather_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    contexts: Res<ActiveInputContexts>,
    mapping_assets: Res<Assets<InputMappingContextAsset>>,
    mut writer: MessageWriter<InputActionEvent>,
) {
    let mut mouse_delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        mouse_delta += motion.delta;
    }

    for handle in &contexts.contexts {
        let Some(ctx) = mapping_assets.get(handle) else {
            continue;
        };

        for mapping in &ctx.mappings {
            let value = evaluate_bindings(&mapping.bindings, &keyboard, &mouse_buttons, mouse_delta);
            if let Some(value) = value {
                writer.write(InputActionEvent {
                    action: InputActionId(mapping.action.clone()),
                    value,
                });
            }
        }
    }
}

fn evaluate_bindings(
    bindings: &[InputBinding],
    keyboard: &ButtonInput<KeyCode>,
    mouse_buttons: &ButtonInput<MouseButton>,
    mouse_delta: Vec2,
) -> Option<InputActionValue> {
    for binding in bindings {
        match binding {
            InputBinding::Wasd => {
                let mut axis = Vec2::ZERO;
                if keyboard.pressed(KeyCode::KeyW) {
                    axis.y += 1.0;
                }
                if keyboard.pressed(KeyCode::KeyS) {
                    axis.y -= 1.0;
                }
                if keyboard.pressed(KeyCode::KeyA) {
                    axis.x -= 1.0;
                }
                if keyboard.pressed(KeyCode::KeyD) {
                    axis.x += 1.0;
                }
                if axis != Vec2::ZERO {
                    return Some(InputActionValue::Axis2D(axis.normalize_or_zero()));
                }
            }
            InputBinding::MouseDelta => {
                if mouse_delta != Vec2::ZERO {
                    return Some(InputActionValue::Axis2D(mouse_delta));
                }
            }
            InputBinding::MouseLeft => {
                return Some(InputActionValue::Digital(
                    mouse_buttons.pressed(MouseButton::Left),
                ));
            }
            InputBinding::KeyboardSpace => {
                return Some(InputActionValue::Digital(keyboard.pressed(KeyCode::Space)));
            }
            InputBinding::KeyboardQ => {
                return Some(InputActionValue::Digital(keyboard.pressed(KeyCode::KeyQ)));
            }
            InputBinding::KeyboardR => {
                return Some(InputActionValue::Digital(keyboard.pressed(KeyCode::KeyR)));
            }
            InputBinding::KeyboardE => {
                return Some(InputActionValue::Digital(keyboard.pressed(KeyCode::KeyE)));
            }
            InputBinding::GamepadLeftStick
            | InputBinding::GamepadRightStick
            | InputBinding::GamepadSouth
            | InputBinding::GamepadRightTrigger
            | InputBinding::GamepadLeftShoulder => {}
        }
    }
    None
}

pub fn axis2d(value: InputActionValue) -> Vec2 {
    match value {
        InputActionValue::Axis2D(v) => v,
        _ => Vec2::ZERO,
    }
}