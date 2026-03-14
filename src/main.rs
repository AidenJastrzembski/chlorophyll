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
// TODO: reapply doesnt do hooks
//
// TODO: add more templates for things that i use
// - things like starship, ghostty,
//
// TODO: dedupe entries in the list command when they exist
// both in the wallpaper dir and the config

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    cli::Cli::parse().run()
}
