use crate::config::{Config, ThemeConfig};
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

pub fn change_theme(
    theme: &Theme,
    config: &Config,
    named: Option<(&str, &ThemeConfig)>,
) -> Result<()> {
    let wallpaper = &theme.wallpaper;
    println!("Wallpaper: {}", wallpaper.display());

    let wallpaper_str = wallpaper.to_string_lossy().to_string();

    // use per-theme override if present, else fall back to global config
    let kill_cmd = named
        .and_then(|(_, tc)| tc.wallpaper_kill.as_deref())
        .or(config.wallpaper_kill.as_deref());
    let wp_cmd = named
        .and_then(|(_, tc)| tc.wallpaper_command.as_deref())
        .or(config.wallpaper_command.as_deref());

    // kill the previous wallpaper daemon if configured
    if let Some(kill_cmd) = kill_cmd {
        if run_sh(kill_cmd).is_ok() {
            println!("Ran wallpaper_kill: {kill_cmd}");
        }
    }

    // set the new wallpaper if configured
    if let Some(wp_cmd) = wp_cmd {
        // only need {{wallpaper}} for this substitution, build a minimal vars map
        let mut vars = std::collections::HashMap::new();
        vars.insert("wallpaper".to_string(), wallpaper_str.clone());
        let resolved = renderer::substitute(wp_cmd, &vars);
        run_sh(&resolved).context("wallpaper_command failed")?;
        println!("Ran wallpaper_command: {resolved}");
    }

    // extract palette once if hooks or templates need it
    let palette = if !config.hooks.is_empty() || !config.templates.is_empty() {
        Some(theme.palette(config.palette_size)?)
    } else {
        None
    };

    // run post-theme-change hooks with full template vars (colors + wallpaper)
    if let Some(ref palette) = palette {
        if !config.hooks.is_empty() {
            let vars = renderer::build_variables(palette, &wallpaper_str);

            for hook in &config.hooks {
                let resolved = renderer::substitute(&hook.command, &vars);
                match run_sh(&resolved) {
                    Ok(()) => println!("Ran hook: {resolved}"),
                    Err(e) => eprintln!("warning: hook failed: {e}"),
                }
            }
        }
    }

    if let Some((name, _)) = named {
        history::save_custom_theme(name)?;
    } else {
        history::save_wallpaper(&theme.wallpaper)?;
    }

    if let Some(ref palette) = palette {
        renderer::render_templates(palette, &theme.wallpaper, &config.templates)?;
    }

    Ok(())
}
