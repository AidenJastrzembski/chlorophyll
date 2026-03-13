use crate::utils::paths;
use crate::utils::PaletteSize;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Deserialize)]
pub struct Template {
    pub name: String,
    pub reload: Option<String>,
}

#[derive(Deserialize)]
pub struct Hook {
    pub command: String,
}

#[derive(Deserialize)]
pub struct ThemeConfig {
    pub path: String,
    pub wallpaper_command: Option<String>,
    pub wallpaper_kill: Option<String>,
}

#[derive(Deserialize)]
pub struct Config {
    pub wallpaper_dir: String,
    pub wallpaper_command: Option<String>,
    pub wallpaper_kill: Option<String>,
    /// number of colors to extract from the wallpaper
    #[serde(default = "default_palette_size")]
    pub palette_size: PaletteSize,
    #[serde(default)]
    pub templates: Vec<Template>,
    /// post-theme-change hooks. can use {{color0}}, {{wallpaper}}, etc.
    /// i.e. setting border colors on your window manager, or wallpaper for your
    /// lock screen
    #[serde(default)]
    pub hooks: Vec<Hook>,
    #[serde(default)]
    pub theme: HashMap<String, ThemeConfig>,
}

// https://serde.rs/attr-default.html
fn default_palette_size() -> PaletteSize {
    PaletteSize::new(16)
}

impl Config {
    /// returns the path to the config file
    pub fn config_path() -> Result<std::path::PathBuf> {
        paths::config_file()
    }

    /// loads the config and returns it
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        // if config file exists, load it
        if config_path.exists() {
            let contents =
                fs::read_to_string(&config_path).context("Failed to read config file")?;
            toml::from_str(&contents).context("Failed to parse config file")
        // else config file doesn't exist, bail with message
        } else {
            bail!(
                "No config file found. Run `chlorophyll init` to create one at\n  {}",
                config_path.display()
            )
        }
    }

    /// initializes a config file with default value and help text
    pub fn init() -> Result<()> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            println!("Config already exists at {}", config_path.display());
            return Ok(());
        }

        let home = std::env::var("HOME").unwrap_or_default();

        let contents = format!(
            r#"# Chlorophyll configuration
            # wallpaper_dir: directory containing your wallpaper images.
            # Supported formats: png, jpg, jpeg, gif, webp

            wallpaper_dir = "{home}/.config/wallpapers"

            # Command to set the wallpaper. {{{{wallpaper}}}} is replaced with the path.
            # wallpaper_command = "swaybg -i {{{{wallpaper}}}}"

            # Optional: kill the previous wallpaper daemon before spawning a new one
            # wallpaper_kill = "pkill swaybg"

            # Number of colors to extract from the wallpaper
            # Remember that if you change this you'll have to change the colorN's that
            # your templates reference
            # palette_size = 16

            # Optional: commands to run after the theme is applied.
            # Uses the same variables as templates: {{{{color0}}}}, {{{{color0.strip}}}}, etc.
            # Named colors: {{{{background}}}}, {{{{foreground}}}}, {{{{primary}}}}, {{{{secondary}}}}
            # Each supports .strip, .rgb, .red, .green, .blue suffixes
            #
            # [[hooks]]
            # command = "riverctl border-color-focused {{{{color0.strip}}}}"

            # Optional: reload hooks for templates
            # Place template files in ~/.config/chlorophyll/templates/
            # Rendered output goes to ~/.cache/chlorophyll/
            #
            # [[templates]]
            # name = "colors-waybar.css"
            # reload = "killall -SIGUSR2 waybar"
            #
            # [[templates]]
            # name = "colors-rofi.rasi"

            # Optional: custom themes with per-theme wallpaper command overrides
            # Useful when some wallpapers need a different tool (e.g. swww for animated)
            #
            # [theme.animated_bg]
            # path = "{home}/.config/wallpapers/animated_bg.gif"
            # wallpaper_command = "swww img {{{{wallpaper}}}}"
            # wallpaper_kill = "pkill swww"
            "#
        );

        if let Some(parent) = config_path.parent() {
            // this creates the dir and all parent dirs if missing
            //
            // (so .config if it doesnt exist, then .config/chlorophyll/)
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        // write the contents to the file
        fs::write(&config_path, contents).context("Failed to write config file")?;
        println!("Created config at {}", config_path.display());

        // create the templates directory, in the config/chlorophyll dir
        let templates_dir = paths::templates_dir()?;
        fs::create_dir_all(&templates_dir).context("Failed to create templates directory")?;

        Ok(())
    }

    /// Append a [[templates]] entry to config.toml as raw text.
    /// This avoids a deserialize/serialize round-trip that would strip comments.
    pub fn append_template_entry(name: &str, reload: Option<&str>) -> Result<()> {
        let config_path = Self::config_path()?;
        // if no config path return early
        if !config_path.exists() {
            bail!("No config file found. Run `chlorophyll init` first.");
        }

        // open the config file and prepare it to be appended
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&config_path)
            .context("Failed to open config file for appending")?;

        let mut entry = format!("\n[[templates]]\nname = \"{name}\"\n");
        // if there is a reload command included, add that too
        if let Some(cmd) = reload {
            entry.push_str(&format!("reload = \"{cmd}\"\n"));
        }

        // append the entry to the file
        file.write_all(entry.as_bytes())
            .context("Failed to append template entry to config")?;

        Ok(())
    }
}
