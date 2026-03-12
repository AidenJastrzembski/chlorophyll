use crate::utils::paths;
use crate::utils::rgb::Rgb;
use anyhow::{Context, Result};
use std::fs;

/// loads the cache for the given hash
pub fn load_cache(hash: &str) -> Result<Option<Vec<Rgb>>> {
    let path = paths::cache_dir()?.join(format!("{hash}.json"));
    // return none if file doesnt exist
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path).context("Failed to read cache file")?;
    let palette: Vec<Rgb> = serde_json::from_str(&data).context("Failed to parse cache file")?;
    Ok(Some(palette))
}

/// saves the cache for the given hash
pub fn save_cache(hash: &str, palette: &[Rgb]) -> Result<()> {
    let dir = paths::cache_dir()?;
    // create the cache dir if it doesnt exist
    fs::create_dir_all(&dir).context("Failed to create cache dir")?;

    let data = serde_json::to_string(&palette)?;
    fs::write(dir.join(format!("{hash}.json")), data).context("Failed to write cache file")?;
    Ok(())
}

/// removes the cache dir
pub fn clear_cache() -> Result<()> {
    let dir = paths::cache_dir()?;
    fs::remove_dir_all(&dir).context("Failed to remove cache dir")?;
    println!("Cache cleared");
    Ok(())
}
