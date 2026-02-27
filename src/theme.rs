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
    /// create the theme being used at runtime
    pub fn new(wallpaper: PathBuf) -> Self {
        Theme { wallpaper }
    }

    /// detected from file extension
    pub fn is_animated(&self) -> bool {
        let ext = match self.wallpaper.extension() {
            Some(ext) => ext.to_str().unwrap(),
            None => return false,
        };
        ANIMATED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    }

    /// sha256 of wallpaper_path + ":" + file_contents
    pub fn hash(&self) -> Result<String> {
        // read the wallpaper file into a vec of u8 (a buffer)
        // NOTE: if large files slow system down or use too much mem (doubt),
        // use a 65535 byte buffer (64k bytes)
        let contents = fs::read(&self.wallpaper).context("Failed to read wallpaper file")?;

        let mut hasher = Sha256::new();
        // update the hasher with the path
        hasher.update(self.wallpaper.to_string_lossy().as_bytes());
        // add the separator
        hasher.update(b":");
        // add the buffer
        hasher.update(&contents);
        // return the hash as String
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// check cache, compute if miss, return scored palette (highest score first)
    pub fn palette(&self) -> Result<Vec<(u8, u8, u8)>> {
        let hash = self.hash()?;

        if let Some(cached) = cache::load_cache(&hash)? {
            return Ok(cached);
        }

        let palette = colors::scored_palette(&self.wallpaper)?;
        cache::save_cache(&hash, &palette)?;
        Ok(palette)
    }

    /// dx helper fn: palette()[0] formatted as "0xRRGGBB"
    pub fn dominant_color(&self) -> Result<String> {
        let palette = self.palette()?;
        let (r, g, b) = palette[0];
        Ok(format!("0x{r:02x}{g:02x}{b:02x}"))
    }
}

/// Try to find the name as a file on the system. if found, return the path as a PathBuf
///
/// for extension type, if name.ext exists in the wallpaper dir, return path else bail with message
pub fn find_wallpaper(dir: &str, name: &str) -> Result<PathBuf> {
    // check if name is a direct path to an existing image file
    let direct = PathBuf::from(name);
    if direct.is_file()
        && direct
            // .extension() returns Option<&OsStr>
            .extension()
            // .and_then() unwraps the Some, applies the 'e.to_str()',
            // and re-wraps the result. If it's None, it returns the None
            .and_then(|e| e.to_str())
            // .is_some_and() returns true only if the Option is Some
            // AND the inner value is true
            .is_some_and(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
    {
        return Ok(direct);
    }

    for ext in IMAGE_EXTENSIONS {
        let path = PathBuf::from(dir).join(format!("{name}.{ext}"));
        if path.exists() {
            return Ok(path);
        }
    }
    bail!("No wallpaper found for '{name}' in {dir}")
}

/// Return all image file paths in `dir`
/// This works but i find it hard to read. might just
/// be a rust skill issue but the zig equiv is much cleaner
pub fn list_wallpapers(dir: &str) -> Result<Vec<PathBuf>> {
    let mut wallpapers: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {dir}"))?
        // filter out non-image files
        .filter_map(|entry| {
            let entry = entry.ok()?; // skip unreadable entries
            let path = entry.path();
            let ext = path.extension()?.to_str()?;

            // check if file extension is in IMAGE_EXTENSIONS
            if IMAGE_EXTENSIONS
                .iter()
                .any(|valid| ext.to_lowercase().ends_with(valid))
            {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    wallpapers.sort();
    Ok(wallpapers)
}
