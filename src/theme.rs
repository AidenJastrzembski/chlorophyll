use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

use crate::utils::{cache, colors};

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];
const ANIMATED_EXTENSIONS: &[&str] = &["gif", "webp", "apng"];

pub struct Theme {
    pub wallpaper: PathBuf,
}

impl Theme {
    pub fn new(wallpaper: PathBuf) -> Self {
        Theme { wallpaper }
    }

    /// Detected from file extension (gif, webp, apng)
    pub fn is_animated(&self) -> bool {
        self.wallpaper
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ANIMATED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// SHA256 of wallpaper_path + ":" + file_contents
    pub fn hash(&self) -> Result<String> {
        let contents = fs::read(&self.wallpaper).context("Failed to read wallpaper file")?;
        let mut hasher = Sha256::new();
        hasher.update(self.wallpaper.to_string_lossy().as_bytes());
        hasher.update(b":");
        hasher.update(&contents);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Check cache, compute if miss, return scored palette (highest score first)
    pub fn palette(&self) -> Result<Vec<(u8, u8, u8)>> {
        let hash = self.hash()?;

        if let Some(cached) = cache::load_cache(&hash)? {
            return Ok(cached);
        }

        let palette = colors::scored_palette(&self.wallpaper)?;
        cache::save_cache(&hash, &palette)?;
        Ok(palette)
    }

    /// Convenience: palette()[0] formatted as "0xRRGGBB"
    pub fn dominant_color(&self) -> Result<String> {
        let palette = self.palette()?;
        let (r, g, b) = palette[0];
        Ok(format!("0x{r:02x}{g:02x}{b:02x}"))
    }
}

/// Scan `dir` for a file matching `name.*` with a supported image extension
pub fn find_wallpaper(dir: &str, name: &str) -> Result<PathBuf> {
    for ext in IMAGE_EXTENSIONS {
        let path = PathBuf::from(dir).join(format!("{name}.{ext}"));
        if path.exists() {
            return Ok(path);
        }
    }
    bail!("No wallpaper found for '{name}' in {dir}")
}

/// Return all image file paths in `dir`
pub fn list_wallpapers(dir: &str) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir).context(format!("Failed to read directory: {dir}"))?;
    let mut wallpapers: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    wallpapers.sort();
    Ok(wallpapers)
}
