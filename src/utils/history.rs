use anyhow::{Context, Result, bail};
use std::fs;
use std::path::PathBuf;

/// returns the path to the history file
fn history_file() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".cache/chlorophyll/last_wallpaper"))
}

/// saves the last applied wallpaper path
/// In the future, might save the last N wallpapers
pub fn save_wallpaper(path: &std::path::Path) -> Result<()> {
    let file = history_file()?;
    fs::create_dir_all(file.parent().unwrap()).context("Failed to create cache dir")?;
    fs::write(&file, path.to_string_lossy().as_bytes())
        .context("Failed to write last wallpaper state")?;
    Ok(())
}

/// loads the last applied wallpaper path
pub fn load_wallpaper() -> Result<PathBuf> {
    let file = history_file()?;
    if !file.exists() {
        bail!("No previous wallpaper found. Apply a theme first.");
    }
    let data = fs::read_to_string(&file).context("Failed to read last wallpaper state")?;
    Ok(PathBuf::from(data.trim()))
}
