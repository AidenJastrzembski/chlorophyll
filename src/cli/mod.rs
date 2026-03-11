mod change_theme;
mod list_themes;
mod preview;

pub(crate) use change_theme::change_theme;
use list_themes::list_themes;
use preview::preview_palette;

use crate::config::Config;
use crate::templates::comptime_templates::{find_comptime_template, list_names};
use crate::theme::{Theme, find_wallpaper};
use crate::utils::cache::clear_cache;
use crate::utils::colors;
use crate::utils::history::reapply_last_wallpaper;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
    /// skip cache and force palette re-extraction
    #[arg(long)]
    no_cache: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create the config file at ~/.config/chlorophyll/config.toml
    Init,
    /// Clear the cache stored in ~/.cache/chlorophyll
    Clear,
    /// List available wallpapers in the wallpapers directory
    List,
    /// Reapply the last used wallpaper theme. Useful for startup sequences
    Reapply,
    /// Apply a theme from a wallpaper in your wallpapers directory
    ///
    /// Usage: chlorophyll from <name>
    From { name: String },
    /// Preview the extracted color palette for a wallpaper
    ///
    /// Usage: chlorophyll preview <name>
    Preview { name: String },
    /// Generate the cache for a wallpaper in your wallpapers directory
    /// without applying it
    ///
    /// Usage: chlorophyll cache <name>
    Cache { name: String },
    /// Write a template to ~/.config/chlorophyll/templates/<name>
    Template {
        name: String,
        /// Force the rewrite of the template, note that this flag skips writing the template into
        /// config, assuming that it is already there
        #[arg(short, long)]
        force: bool,
    },
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            // Commands that dont require the config to be loaded
            Command::Init => {
                Config::init()?;
            }
            Command::Clear => {
                clear_cache()?;
            }
            Command::Template { name, force } => match find_comptime_template(&name) {
                Some(comptime_template) => comptime_template.install(force)?,
                None => {
                    let available = list_names().join(", ");
                    anyhow::bail!("Unknown template '{name}'. Available starters: {available}");
                }
            },
            // Commands that do require the config to be loaded
            command => {
                let config = Config::load()?;
                match command {
                    Command::Reapply => {
                        reapply_last_wallpaper(&config, self.no_cache)?;
                    }
                    Command::List => {
                        if let Some(name) = list_themes(&config.wallpaper_dir, config.palette_size)?
                        {
                            let path = find_wallpaper(&config.wallpaper_dir, &name)?;
                            let mut theme = Theme::new(path);
                            if self.no_cache {
                                theme = theme.skip_cache();
                            }
                            change_theme(&theme, &config)?;
                        }
                    }
                    Command::From { name } => {
                        let path = find_wallpaper(&config.wallpaper_dir, &name)?;
                        let mut theme = Theme::new(path);
                        if self.no_cache {
                            theme = theme.skip_cache();
                        }
                        change_theme(&theme, &config)?;
                    }
                    Command::Preview { name } => {
                        let path = find_wallpaper(&config.wallpaper_dir, &name)?;
                        let mut theme = Theme::new(path);
                        if self.no_cache {
                            theme = theme.skip_cache();
                        }
                        let palette = theme.palette(config.palette_size)?;
                        let labels = colors::assign_labels(&palette);
                        preview_palette(&palette, &name, &labels)?;
                    }
                    Command::Cache { name } => {
                        let path = find_wallpaper(&config.wallpaper_dir, &name)?;
                        let mut theme = Theme::new(path);
                        if self.no_cache {
                            theme = theme.skip_cache();
                        }
                        // generating the palette will cache the results
                        theme.palette(config.palette_size)?;
                    }
                    Command::Init | Command::Clear | Command::Template { .. } => unreachable!(),
                }
            }
        }

        Ok(())
    }
}
