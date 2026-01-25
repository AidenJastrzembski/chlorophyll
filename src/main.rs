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

// NOTE: impls add fns to structs
struct Theme {
    wallpaper: &'static str,
    border_color_focused: &'static str,
    is_animated: bool,
}

const ZEN: Theme = Theme {
    wallpaper: "zen.jpg",
    border_color_focused: "0x67c77c",
    is_animated: false,
};

const HIDEOUT: Theme = Theme {
    wallpaper: "hideout.png",
    border_color_focused: "0x8ce8ff",
    is_animated: false,
};

const FREAK: Theme = Theme {
    wallpaper: "freak.jpg",
    border_color_focused: "0xff79c6",
    is_animated: false,
};

const BLEAK: Theme = Theme {
    wallpaper: "bleak.gif",
    border_color_focused: "0xc9c9c9",
    is_animated: true,
};

static WALLPAPER_ROOT: &str = "/home/aiden/.config/wallpapers/";

fn main() {
    let args = Args::parse();

    if args.list {
        list_themes();
        return;
    }

    if let Some(theme) = args.theme {
        match theme.as_str() {
            "hideout" => change_theme(HIDEOUT),
            "hatsune" => todo!(),
            "zen" => change_theme(ZEN),
            "freak" => change_theme(FREAK),
            "bleak" => change_theme(BLEAK),
            _ => print!("Unknown theme: {}\n", theme),
        }
    }
}

fn list_themes() {
    print!("Available themes:\n\n");
    print!("\tZen - Dream bedroom, kanagawa\n");
    print!("\tHideout - Cool hideout, nostalgic, rose-pine-moon\n");
    print!("\tFreak - Girl drinking energy drinks, tokyonight-storm\n");
    print!("\tBleak - Lonely animated town, vague\n");
}

fn change_theme(theme: Theme) {
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
            .unwrap();
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
            .unwrap();
        println!("Spawned swaybg");
    }

    // change focused border color
    Command::new("riverctl")
        .arg("border-color-focused")
        .arg(theme.border_color_focused)
        .output()
        .unwrap();
}
