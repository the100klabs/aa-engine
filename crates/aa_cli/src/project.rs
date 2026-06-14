use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const REQUIRED_CONFIG_FILES: &[&str] = &["engine.toml", "game.toml"];

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("aa.project.toml not found at {0}")]
    ManifestMissing(PathBuf),
    #[error("failed to read manifest: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse manifest: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectManifest {
    pub schema_version: u32,
    pub name: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub version: String,
    pub engine: EngineSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineSection {
    pub config_root: String,
    pub assets_root: String,
    #[serde(default)]
    pub default_experience: Option<String>,
    #[serde(default)]
    pub startup_scene: Option<String>,
}

pub fn manifest_path(project_root: &Path) -> PathBuf {
    project_root.join("aa.project.toml")
}

pub fn resolve_project_root(path: &Path) -> Result<PathBuf, ProjectError> {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if path.join("aa.project.toml").is_file() {
        return Ok(path);
    }
    if path.file_name().is_some_and(|n| n == "aa.project.toml") {
        return Ok(path.parent().unwrap_or(&path).to_path_buf());
    }
    Err(ProjectError::ManifestMissing(manifest_path(&path)))
}

pub fn load_manifest(project_root: &Path) -> Result<ProjectManifest, ProjectError> {
    let path = manifest_path(project_root);
    let text = std::fs::read_to_string(&path)?;
    let manifest: ProjectManifest = toml::from_str(&text)?;
    Ok(manifest)
}
