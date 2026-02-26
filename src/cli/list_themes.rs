use crate::theme::{Theme, list_wallpapers};
use anyhow::Result;

pub fn list_themes(wallpaper_dir: &str) -> Result<()> {
    let wallpapers = list_wallpapers(wallpaper_dir)?;
    if wallpapers.is_empty() {
        println!("No wallpapers found in {wallpaper_dir}");
        return Ok(());
    }
    println!("Available wallpapers:\n");
    for path in wallpapers {
        let theme = Theme::new(path.clone());
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
        if theme.is_animated() {
            println!("\t{stem} (animated)");
        } else {
            println!("\t{stem}");
        }
    }
    Ok(())
}
