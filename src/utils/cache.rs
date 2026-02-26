use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// returns the path to the cache dir
fn cache_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".cache/chlorophyll"))
}

/// loads the cache for the given hash
pub fn load_cache(hash: &str) -> Result<Option<Vec<(u8, u8, u8)>>> {
    let path = cache_dir()?.join(format!("{hash}.json"));
    // return none if file doesnt exist
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path).context("Failed to read cache file")?;
    let tuples: Vec<[u8; 3]> = serde_json::from_str(&data).context("Failed to parse cache file")?;
    // collect all tuples into a vec and return
    Ok(Some(
        tuples.into_iter().map(|[r, g, b]| (r, g, b)).collect(),
    ))
}

/// saves the cache for the given hash
pub fn save_cache(hash: &str, palette: &[(u8, u8, u8)]) -> Result<()> {
    let dir = cache_dir()?;
    // create the cache dir if it doesnt exist
    fs::create_dir_all(&dir).context("Failed to create cache dir")?;

    // collect all tuples into a vec
    let tuples: Vec<[u8; 3]> = palette.iter().map(|&(r, g, b)| [r, g, b]).collect();
    // convert vec to json for caching
    let data = serde_json::to_string(&tuples)?;
    fs::write(dir.join(format!("{hash}.json")), data).context("Failed to write cache file")?;
    Ok(())
}

/// removes the cache dir
pub fn clear_cache() -> Result<()> {
    let dir = cache_dir()?;
    fs::remove_dir_all(&dir).context("Failed to remove cache dir")?;
    println!("Cache cleared");
    Ok(())
}
