use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub wallpaper_dir: String,
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".config/chlorophyll/config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents =
                fs::read_to_string(&config_path).context("Failed to read config file")?;
            toml::from_str(&contents).context("Failed to parse config file")
        } else {
            bail!(
                "No config file found. Run `chlorophyll init` to create one at\n  {}",
                config_path.display()
            )
        }
    }

    pub fn init() -> Result<()> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            println!("Config already exists at {}", config_path.display());
            return Ok(());
        }

        let home = std::env::var("HOME").context("HOME not set")?;

        let contents = format!(
            r#"# Chlorophyll configuration
#
# wallpaper_dir: directory containing your wallpaper images.
# Supported formats: png, jpg, jpeg, gif, webp
#
# Usage:
#   chlorophyll zen          # finds zen.* in wallpaper_dir
#   chlorophyll -i /any/path # use an arbitrary image
#   chlorophyll -l           # list all wallpapers in wallpaper_dir

wallpaper_dir = "{home}/.config/wallpapers"
"#
        );

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        fs::write(&config_path, contents).context("Failed to write config file")?;
        println!("Created config at {}", config_path.display());

        Ok(())
    }
}
