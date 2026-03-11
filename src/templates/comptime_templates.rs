use crate::config::Config;
use crate::templates::renderer::templates_dir;
use anyhow::{Context, Result, bail};
use std::fs;

/// ComptimeTemplates are like starter templates for different apps for
/// UX. I know comptime is a bit zig-pilled but its what clicks in my head
pub struct ComptimeTemplate {
    pub name: &'static str,
    pub filename: &'static str,
    pub content: &'static str,
    pub reload: Option<&'static str>,
}

/// This is where you actually put the templates which are included
/// into the bundle
pub static COMPTIME_TEMPLATES: &[ComptimeTemplate] = &[
    ComptimeTemplate {
        name: "waybar",
        filename: "colors-waybar.css",
        content: include_str!("comptime_templates/waybar.css"),
        reload: Some("killall -SIGUSR2 waybar"),
    },
    ComptimeTemplate {
        name: "rofi",
        filename: "colors-rofi.rasi",
        // include_str is a macro which reads a file into a string which
        // is included into the program at comptime
        content: include_str!("comptime_templates/rofi.rasi"),
        reload: None,
    },
];

/// find the comptime template which has the same name
pub fn find_comptime_template(name: &str) -> Option<&'static ComptimeTemplate> {
    COMPTIME_TEMPLATES.iter().find(|s| s.name == name)
}

/// list comptime template options
pub fn list_names() -> Vec<&'static str> {
    COMPTIME_TEMPLATES.iter().map(|s| s.name).collect()
}

impl ComptimeTemplate {
    /// Write the starter template file and append a [[templates]] entry to config.toml.
    pub fn install(&self, force: bool) -> Result<()> {
        let dir = templates_dir()?;
        fs::create_dir_all(&dir).context("Failed to create templates directory")?;

        let dest = dir.join(self.filename);
        if dest.exists() && !force {
            bail!(
                "Template already exists at {}\n  Use --force or -f if you want to reinstall.",
                dest.display()
            );
        }

        // write the templates content to the file
        fs::write(&dest, self.content)
            .with_context(|| format!("Failed to write template to {}", dest.display()))?;

        // append a [[templates]] entry to config.toml

        if !force {
            // skip writing it if forcing, assume its already in config
            Config::append_template_entry(self.filename, self.reload)?;
        }

        println!("Installed template to {}", dest.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_known_template() {
        let t = find_comptime_template("waybar");
        assert!(t.is_some());
        let t = t.unwrap();
        assert_eq!(t.name, "waybar");
        assert_eq!(t.filename, "colors-waybar.css");
    }

    #[test]
    fn find_unknown_template() {
        assert!(find_comptime_template("nonexistent").is_none());
    }
}
