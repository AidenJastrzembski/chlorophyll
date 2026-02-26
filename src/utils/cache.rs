use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn cache_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".cache/chlorophyll"))
}

pub fn load_cache(hash: &str) -> Result<Option<Vec<(u8, u8, u8)>>> {
    let path = cache_dir()?.join(format!("{hash}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path).context("Failed to read cache file")?;
    let tuples: Vec<[u8; 3]> =
        serde_json::from_str(&data).context("Failed to parse cache file")?;
    Ok(Some(tuples.into_iter().map(|[r, g, b]| (r, g, b)).collect()))
}

pub fn save_cache(hash: &str, palette: &[(u8, u8, u8)]) -> Result<()> {
    let dir = cache_dir()?;
    fs::create_dir_all(&dir).context("Failed to create cache dir")?;
    let tuples: Vec<[u8; 3]> = palette.iter().map(|&(r, g, b)| [r, g, b]).collect();
    let data = serde_json::to_string(&tuples)?;
    fs::write(dir.join(format!("{hash}.json")), data).context("Failed to write cache file")?;
    Ok(())
}
