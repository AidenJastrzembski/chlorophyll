mod cli;
mod config;
mod theme;
mod utils;

use crate::cli::{change_theme, list_themes};
use crate::config::Config;
use crate::theme::{Theme, find_wallpaper};
use crate::utils::cache::clear_cache;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// TODO:  create preview command which displays the palette (and potentially the wallpaper)
// either in ratatui or using ascii characters
//
// TODO: refactor all/most commands to be subcommands
//
// TODO: add --no-cache flag
//
// TODO: list command should be a ratatui interactive screen with a searchable list, which
// displays the name, and a color palette preview
//
// TODO: build in some default templates for common tools like waybar and rofi (think like pywal)
//
// TODO: build in a templating system which allows users to create templates which will
// apply theme changes to different tools
//
// TODO: build out custom color-thief implementation
//
// TODO: reapply command which allows users to reapply the last theme, usefull for
// startup sequences

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Wallpaper name (e.g. "zen" finds zen.* in wallpaper dir)
    wallpaper: Option<String>,

    /// Path to a specific image file
    #[arg(short, long)]
    image: Option<PathBuf>,

    /// List available wallpapers
    #[arg(short, long)]
    list: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create the config file at ~/.config/chlorophyll/config.toml
    Init,
    /// Clear the cache stored in ~/.cache/chlorophyll
    Clear,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(Command::Init) = args.command {
        return Config::init();
    }

    if let Some(Command::Clear) = args.command {
        return clear_cache();
    }

    let config = Config::load()?;

    if args.list {
        list_themes(&config.wallpaper_dir)?;
        return Ok(());
    }

    if let Some(path) = args.image {
        let theme = Theme::new(path);
        change_theme(&theme)?;
        return Ok(());
    }

    if let Some(name) = args.wallpaper {
        let path = find_wallpaper(&config.wallpaper_dir, &name)?;
        let theme = Theme::new(path);
        change_theme(&theme)?;
        return Ok(());
    }
    Ok(())
}
