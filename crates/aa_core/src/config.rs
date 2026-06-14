use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::de::DeserializeOwned;

use crate::error::ConfigError;
use crate::paths::project_root;

const ENGINE_BASE_TOML: &str = include_str!("engine_base.toml");

const CONFIG_FILES: &[&str] = &[
    "config/engine.toml",
    "config/game.toml",
    "config/scalability.toml",
    "config/user.toml",
];

/// Layered TOML configuration merged from engine defaults and project files.
#[derive(Resource, Debug, Clone)]
pub struct ConfigProvider {
    project_root: PathBuf,
    file_layers: toml::Table,
    cli_overrides: toml::Table,
}

impl ConfigProvider {
    pub fn load(project_root: &Path) -> Result<Self, ConfigError> {
        let mut file_layers = parse_toml_str(ENGINE_BASE_TOML, "engine_base")?;

        for relative in CONFIG_FILES {
            merge_optional_file(&mut file_layers, &project_root.join(relative))?;
        }

        Ok(Self {
            project_root: project_root.to_path_buf(),
            file_layers,
            cli_overrides: toml::Table::new(),
        })
    }

    pub fn load_from_env() -> Result<Self, ConfigError> {
        Self::load(&project_root())
    }

    /// Fallback when project config files are missing or invalid.
    pub fn with_engine_defaults(project_root: PathBuf) -> Self {
        Self {
            project_root,
            file_layers: parse_toml_str(ENGINE_BASE_TOML, "engine_base")
                .expect("engine_base.toml must parse"),
            cli_overrides: toml::Table::new(),
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let merged = self.effective_table();
        let value = get_at_dot_path(&merged, key)?.clone();
        T::deserialize(value).ok()
    }

    pub fn apply_cli_override(&mut self, key: &str, value: &str) -> Result<(), ConfigError> {
        let parsed: toml::Value = if let Ok(integer) = value.parse::<i64>() {
            toml::Value::Integer(integer)
        } else if let Ok(float) = value.parse::<f64>() {
            toml::Value::Float(float)
        } else if let Ok(boolean) = value.parse::<bool>() {
            toml::Value::Boolean(boolean)
        } else {
            value.parse().map_err(|source| ConfigError::Parse {
                key: key.to_owned(),
                source: Box::new(source),
            })?
        };
        set_at_dot_path(&mut self.cli_overrides, key, parsed);
        Ok(())
    }

    fn effective_table(&self) -> toml::Table {
        let mut merged = self.file_layers.clone();
        merge_tables(&mut merged, self.cli_overrides.clone());
        merged
    }
}

fn parse_toml_str(content: &str, label: &str) -> Result<toml::Table, ConfigError> {
    let value: toml::Value = toml::from_str(content).map_err(|source| ConfigError::Parse {
        key: label.to_owned(),
        source: Box::new(source),
    })?;
    value.as_table().cloned().ok_or_else(|| ConfigError::Parse {
        key: label.to_owned(),
        source: Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected TOML table at root",
        )),
    })
}

fn merge_optional_file(merged: &mut toml::Table, path: &Path) -> Result<(), ConfigError> {
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)?;
    let overlay = parse_toml_str(&content, &path.display().to_string())?;
    merge_tables(merged, overlay);
    Ok(())
}

fn merge_tables(base: &mut toml::Table, overlay: toml::Table) {
    for (key, value) in overlay {
        match (base.get(&key), value) {
            (Some(toml::Value::Table(base_sub)), toml::Value::Table(overlay_sub)) => {
                let mut sub = base_sub.clone();
                merge_tables(&mut sub, overlay_sub);
                base.insert(key, toml::Value::Table(sub));
            }
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
}

fn get_at_dot_path<'a>(table: &'a toml::Table, path: &str) -> Option<&'a toml::Value> {
    let mut current = table;
    let segments: Vec<&str> = path.split('.').collect();
    for (i, segment) in segments.iter().enumerate() {
        let next = current.get(*segment)?;
        if i == segments.len() - 1 {
            return Some(next);
        }
        current = next.as_table()?;
    }
    None
}

fn set_at_dot_path(table: &mut toml::Table, path: &str, value: toml::Value) {
    let segments: Vec<&str> = path.split('.').collect();
    if segments.is_empty() {
        return;
    }
    if segments.len() == 1 {
        table.insert(segments[0].to_owned(), value);
        return;
    }

    let head = segments[0];
    let tail = segments[1..].join(".");
    let mut sub = table
        .get(head)
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_default();

    set_at_dot_path(&mut sub, &tail, value);
    table.insert(head.to_owned(), toml::Value::Table(sub));
}
