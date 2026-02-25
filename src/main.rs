mod utils;
use anyhow::{Context, Result};
use clap::Parser;
use std::process::{Command, Stdio};
use utils::colors;

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

const ZEN: Theme = Theme {
    wallpaper: "zen.jpg",
    is_animated: false,
};

const HIDEOUT: Theme = Theme {
    wallpaper: "hideout.png",
    is_animated: false,
};

const FREAK: Theme = Theme {
    wallpaper: "freak.jpg",
    is_animated: false,
};

const BLEAK: Theme = Theme {
    wallpaper: "bleak.gif",
    is_animated: true,
};

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

fn list_themes() {
    println!("Available themes:\n");
    println!("\tZen - Dream bedroom, kanagawa");
    println!("\tHideout - Cool hideout, nostalgic, rose-pine-moon");
    println!("\tFreak - Girl drinking energy drinks, tokyonight-storm");
    println!("\tBleak - Lonely animated town, vague");
}

fn change_theme(theme: Theme) -> Result<()> {
    Command::new("pkill").arg("swaybg").status().ok();
    println!("Killed swaybg");

    // see which one we actually need
    Command::new("pkill")
        .arg("-9")
        .arg("swww-daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null()) // tell swww to shut the fuck up
        .stderr(Stdio::null())
        .status()
        .ok();
    println!("Killed swww");

    let wallpaper = WALLPAPER_ROOT.to_owned() + &theme.wallpaper;
    println!("Wallpaper: {}", wallpaper);

    if theme.is_animated {
        Command::new("swww-daemon")
            .stdin(Stdio::null())
            .stdout(Stdio::null()) // tell swww to shut the fuck up
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to run swww-daemon")?;

        Command::new("swww")
            .arg("img")
            .arg(wallpaper)
            .stdin(Stdio::null())
            .stdout(Stdio::null()) // tell swww to shut the fuck up
            .stderr(Stdio::null())
            .status()
            .unwrap();

        println!("Spawned swww");
    } else {
        Command::new("swaybg")
            .arg("-i")
            .arg(wallpaper)
            .stdout(Stdio::null()) // tell swaybg to shut the fuck up
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to run swaybg")?;
        println!("Spawned swaybg");
    }

    // change focused border color
    println!("Changing focused border color to: {}", theme.color());
    Command::new("riverctl")
        .arg("border-color-focused")
        .arg(theme.color())
        .output()
        .unwrap();

    Ok(())
}
