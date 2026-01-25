use clap::Parser;
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    theme: Option<String>,

    #[arg(short, long, default_value_t = false)]
    list: bool,
}

static WALLPAPER_ROOT: &str = "/home/aiden/.config/wallpapers/";

fn main() {
    let args = Args::parse();

    if args.list {
        list_themes();
        return;
    }

    if let Some(theme) = args.theme {
        match theme.as_str() {
            "hideout" => change_to_hideout(),
            "hatsune" => todo!(),
            "zen" => change_to_zen(),
            "freak" => change_to_freak(),
            _ => print!("Unknown theme: {}\n", theme),
        }
    }
}

fn list_themes() {
    print!("Available themes:\n\n");
    print!("\tZen - Dream bedroom, kanagawa\n");
    print!("\tHideout - Cool hideout, nostalgic, rose-pine-moon\n");
    print!("\tFreak - Girl drinking energy drinks, tokyonight-storm\n");
}

fn change_to_freak() {
    Command::new("pkill").arg("swaybg").status().ok();
    println!("Killed swaybg");

    let wallpaper = WALLPAPER_ROOT.to_owned() + "freak.jpg";
    println!("Wallpaper: {}", wallpaper);

    Command::new("swaybg")
        .arg("-i")
        .arg(wallpaper)
        .stdout(Stdio::null()) // tell swaybg to shut the fuck up
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    println!("Spawned swaybg");

    // change border color to a magenta
    Command::new("riverctl")
        .arg("border-color-focused")
        .arg("0xff79c6")
        .output()
        .unwrap();
}

fn change_to_zen() {
    Command::new("pkill").arg("swaybg").status().ok();
    println!("Killed swaybg");

    let wallpaper = WALLPAPER_ROOT.to_owned() + "zen.jpg";
    println!("Wallpaper: {}", wallpaper);

    Command::new("swaybg")
        .arg("-i")
        .arg(wallpaper)
        .stdout(Stdio::null()) // tell swaybg to shut the fuck up
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    println!("Spawned swaybg");

    // change border color to a forest green
    Command::new("riverctl")
        .arg("border-color-focused")
        .arg("0x67c77c")
        .output()
        .unwrap();

    // could be worth changing unfocused color too
}

fn change_to_hideout() {
    Command::new("pkill").arg("swaybg").status().ok();
    println!("Killed swaybg");

    let wallpaper = WALLPAPER_ROOT.to_owned() + "hideout.png";
    println!("Wallpaper: {}", wallpaper);

    Command::new("swaybg")
        .arg("-i")
        .arg(wallpaper)
        .stdout(Stdio::null()) // tell swaybg to shut the fuck up
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    println!("Spawned swaybg");

    // change border color to a sky blue
    Command::new("riverctl")
        .arg("border-color-focused")
        .arg("0x8ce8ff")
        .output()
        .unwrap();
}
