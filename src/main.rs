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

    Command::new("swaybg")
        .arg("-i")
        .arg("/home/aiden/.config/wallpapers/freak.jpg")
        .stdout(Stdio::null()) // tell swaybg to shut the fuck up
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    println!("Spawned swaybg");
}

fn change_to_zen() {
    Command::new("pkill").arg("swaybg").status().ok();
    println!("Killed swaybg");

    Command::new("swaybg")
        .arg("-i")
        .arg("/home/aiden/.config/wallpapers/zen.jpg")
        .stdout(Stdio::null()) // tell swaybg to shut the fuck up
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    println!("Spawned swaybg");
}

fn change_to_hideout() {
    Command::new("pkill").arg("swaybg").status().ok();
    println!("Killed swaybg");

    Command::new("swaybg")
        .arg("-i")
        .arg("/home/aiden/.config/wallpapers/hideout.png")
        .stdout(Stdio::null()) // tell swaybg to shut the fuck up
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    println!("Spawned swaybg");
}
