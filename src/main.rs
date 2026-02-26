mod cli;
mod config;
mod theme;
mod utils;

use crate::cli::{change_theme, list_themes};
use crate::config::Config;
use crate::theme::{Theme, find_wallpaper};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(Command::Init) = args.command {
        return Config::init();
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

    // no args: print help
    use clap::CommandFactory;
    Args::command().print_help()?;
    println!();
    Ok(())
}
