mod cli;
mod config;
mod templates;
mod theme;
mod utils;

// TODO: write tests for all commands
//
// TODO: Implement a kmeans stategy to better pick out colors from wallpapers
//
// TODO: find some way to allow user to specify wallpaper command overrides per theme...
// i.e. user uses swaybg for wallpapers, but animated ones use swww

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    cli::Cli::parse().run()
}
