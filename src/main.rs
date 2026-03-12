mod cli;
mod config;
mod templates;
mod theme;
mod utils;

// TODO: write tests for all commands
//
// TODO: Implement a kmeans stategy to better pick out colors from wallpapers
//
// TODO: truly silence the output. we dont need to print much to the user at all we just need
// it to work
//
// TODO: add more templates for things that i use

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    cli::Cli::parse().run()
}
