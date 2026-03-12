use crate::cli::change_theme;
use crate::config::Config;
use crate::theme::Theme;
use crate::utils::paths;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::PathBuf;

const THEME_PREFIX: &str = "theme:";

/// returns the path to the history file
fn history_file() -> Result<PathBuf> {
    Ok(paths::cache_dir()?.join("last_theme"))
}

fn write_history(content: &str) -> Result<()> {
    let file = history_file()?;
    fs::create_dir_all(file.parent().unwrap()).context("Failed to create cache dir")?;
    fs::write(&file, content).context("Failed to write last wallpaper state")?;
    Ok(())
}

/// saves the last applied wallpaper path
pub fn save_wallpaper(path: &std::path::Path) -> Result<()> {
    write_history(&path.to_string_lossy())
}

/// saves a custom theme entry (prefixed with "theme:")
pub fn save_custom_theme(name: &str) -> Result<()> {
    write_history(&format!("{THEME_PREFIX}{name}"))
}

/// loads the raw history string
fn load_history() -> Result<String> {
    let file = history_file()?;
    if !file.exists() {
        bail!("No previous wallpaper found. Apply a theme first.");
    }
    let data = fs::read_to_string(&file).context("Failed to read last wallpaper state")?;
    Ok(data.trim().to_string())
}

pub fn reapply_last_wallpaper(config: &Config, force: bool) -> Result<()> {
    let entry = load_history()?;

    if let Some(name) = entry.strip_prefix(THEME_PREFIX) {
        // named theme — look it up in config
        let tc = config
            .theme
            .get(name)
            .with_context(|| format!("Named theme '{name}' not found in config"))?;
        let mut theme = Theme::new(PathBuf::from(&tc.path));
        if force {
            theme = theme.skip_cache();
        }
        change_theme(&theme, config, Some((name, tc)))?;
    } else {
        // plain wallpaper path
        let mut theme = Theme::new(PathBuf::from(&entry));
        if force {
            theme = theme.skip_cache();
        }
        change_theme(&theme, config, None)?;
    }

    Ok(())
}
