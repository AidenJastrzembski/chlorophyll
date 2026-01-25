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
}

const ZEN: Theme = Theme {
    wallpaper: "zen.jpg",
    border_color_focused: "0x67c77c",
};

const HIDEOUT: Theme = Theme {
    wallpaper: "hideout.png",
    border_color_focused: "0x8ce8ff",
};

const FREAK: Theme = Theme {
    wallpaper: "freak.jpg",
    border_color_focused: "0xff79c6",
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

fn change_theme(theme: Theme) {
    Command::new("pkill").arg("swaybg").status().ok();
    println!("Killed swaybg");

    let wallpaper = WALLPAPER_ROOT.to_owned() + &theme.wallpaper;
    println!("Wallpaper: {}", wallpaper);

    Command::new("swaybg")
        .arg("-i")
        .arg(wallpaper)
        .stdout(Stdio::null()) // tell swaybg to shut the fuck up
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    println!("Spawned swaybg");

    // change focused border color
    Command::new("riverctl")
        .arg("border-color-focused")
        .arg(theme.border_color_focused)
        .output()
        .unwrap();
}
