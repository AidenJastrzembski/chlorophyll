use crate::Theme;
use crate::WALLPAPER_ROOT;
use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub fn change_theme(theme: Theme) -> Result<()> {
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

    let wallpaper = WALLPAPER_ROOT.to_owned() + theme.wallpaper;
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
