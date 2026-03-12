mod cli;
mod config;
mod templates;
mod theme;
mod utils;

// TODO: write tests for all commands
//
// TODO: Implement a kmeans stategy to better pick out colors from wallpapers
//
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    cli::Cli::parse().run()
}
