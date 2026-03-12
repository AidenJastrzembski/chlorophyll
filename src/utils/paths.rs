use anyhow::{Context, Result};
use std::path::PathBuf;

/// ~/.config/chlorophyll or $XDG_CONFIG_HOME/chlorophyll
pub fn config_dir() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("chlorophyll"));
    }
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".config/chlorophyll"))
}

/// ~/.cache/chlorophyll or $XDG_CACHE_HOME/chlorophyll
pub fn cache_dir() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(xdg).join("chlorophyll"));
    }
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".cache/chlorophyll"))
}

/// config_dir()/config.toml
pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// config_dir()/templates
pub fn templates_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("templates"))
}
