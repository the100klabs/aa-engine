use bevy::prelude::*;
use std::collections::HashMap;

/// Float attributes with base + current values (GAS AttributeSet equivalent).
#[derive(Component, Debug, Clone, Default)]
pub struct AttributeSet {
    base: HashMap<String, f32>,
    current: HashMap<String, f32>,
    mins: HashMap<String, f32>,
    maxs: HashMap<String, f32>,
}

impl AttributeSet {
    pub fn insert_attribute(&mut self, name: impl Into<String>, default: f32, min: f32, max: f32) {
        let name = name.into();
        self.base.insert(name.clone(), default);
        self.current.insert(name.clone(), default);
        self.mins.insert(name.clone(), min);
        self.maxs.insert(name, max);
    }

    pub fn get(&self, name: &str) -> Option<f32> {
        self.current.get(name).copied()
    }

    pub fn set_current(&mut self, name: &str, value: f32) {
        if let Some(current) = self.current.get_mut(name) {
            let min = self.mins.get(name).copied().unwrap_or(f32::MIN);
            let max = self.maxs.get(name).copied().unwrap_or(f32::MAX);
            *current = value.clamp(min, max);
        }
    }

    pub fn apply_modifier(&mut self, name: &str, op: crate::ModifierOp, magnitude: f32) {
        let Some(current) = self.current.get_mut(name) else {
            return;
        };
        match op {
            crate::ModifierOp::Add => *current += magnitude,
            crate::ModifierOp::Multiply => *current *= magnitude,
            crate::ModifierOp::Override => *current = magnitude,
        }
        let min = self.mins.get(name).copied().unwrap_or(f32::MIN);
        let max = self.maxs.get(name).copied().unwrap_or(f32::MAX);
        *current = current.clamp(min, max);
    }
}
