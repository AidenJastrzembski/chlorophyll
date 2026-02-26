mod cli;
mod consts;
mod utils;
use crate::cli::{change_theme, list_themes};
use crate::consts::{BLEAK, FREAK, HIDEOUT, ZEN};
use crate::utils::colors;
use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    theme: Option<String>,

    #[arg(short, long, default_value_t = false)]
    list: bool,
}

struct Theme {
    wallpaper: &'static str,
    is_animated: bool,
}

impl Theme {
    pub fn color(&self) -> String {
        println!("wallpaper: {}", self.wallpaper);
        let wall_path = WALLPAPER_ROOT.to_owned() + self.wallpaper;
        let rbg = colors::dominant_color(wall_path.as_str())
            .context("failed to get color")
            .unwrap();
        format!("0x{:02x}{:02x}{:02x}", rbg.0, rbg.1, rbg.2)
    }
}

// TODO: should be defined by user in their config
static WALLPAPER_ROOT: &str = "/home/aiden/.config/wallpapers/";

fn main() -> Result<()> {
    let args = Args::parse();

    if args.list {
        list_themes();
        return Ok(());
    }

    if let Some(theme) = args.theme {
        match theme.as_str() {
            "hideout" => change_theme(HIDEOUT)?,
            "hatsune" => todo!(),
            "zen" => change_theme(ZEN)?,
            "freak" => change_theme(FREAK)?,
            "bleak" => change_theme(BLEAK)?,
            _ => println!("Unknown theme: {}", theme),
        }
    }
    Ok(())
}
