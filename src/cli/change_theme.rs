use crate::config::Config;
use crate::templates::renderer;
use crate::theme::Theme;
use crate::utils::history;
use anyhow::{Context, Result};
use std::process::Command;

/// run a shell command with sh -c
fn run_sh(command: &str) -> Result<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .with_context(|| format!("Failed to run: {command}"))?;

    if !status.success() {
        eprintln!("warning: command exited with {status}: {command}");
    }

    Ok(())
}

pub fn change_theme(theme: &Theme, config: &Config) -> Result<()> {
    let wallpaper = &theme.wallpaper;
    println!("Wallpaper: {}", wallpaper.display());

    let wallpaper_str = wallpaper.to_string_lossy().to_string();

    // kill the previous wallpaper daemon if configured
    if let Some(ref kill_cmd) = config.wallpaper_kill {
        run_sh(kill_cmd).ok();
        println!("Ran wallpaper_kill: {kill_cmd}");
    }

    // set the new wallpaper if configured
    if let Some(ref wp_cmd) = config.wallpaper_command {
        // only need {{wallpaper}} for this substitution, build a minimal vars map
        let mut vars = std::collections::HashMap::new();
        vars.insert("wallpaper".to_string(), wallpaper_str.clone());
        let resolved = renderer::substitute(wp_cmd, &vars);
        run_sh(&resolved).context("wallpaper_command failed")?;
        println!("Ran wallpaper_command: {resolved}");
    }

    // run post-theme-change hooks with full template vars (colors + wallpaper)
    if !config.hooks.is_empty() {
        let palette = theme.palette(config.palette_size)?;
        let vars = renderer::build_variables(&palette, &wallpaper_str);

        for hook in &config.hooks {
            let resolved = renderer::substitute(&hook.command, &vars);
            match run_sh(&resolved) {
                Ok(()) => println!("Ran hook: {resolved}"),
                Err(e) => eprintln!("warning: hook failed: {e}"),
            }
        }
    }

    history::save_wallpaper(&theme.wallpaper)?;

    renderer::render_templates(theme, &config.templates, config.palette_size)?;

    Ok(())
}
