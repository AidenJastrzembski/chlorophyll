use crate::config::Template;
use crate::utils::colors;
use crate::utils::paths;
use crate::utils::rgb::Rgb;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// make all color format variants for a given prefix (hex, strip, rgb, red, green, blue).
fn insert_color_vars(vars: &mut HashMap<String, String>, prefix: &str, c: &Rgb) {
    // Base css/gtk hex code
    vars.insert(prefix.to_string(), c.hex());
    // Some tools want hex but without the #
    vars.insert(
        format!("{prefix}.strip"),
        format!("{:02x}{:02x}{:02x}", c.0, c.1, c.2),
    );
    // CSS rgb values
    vars.insert(format!("{prefix}.rgb"), format!("{},{},{}", c.0, c.1, c.2));
    // float based channels for tools like sway which want 0-1
    vars.insert(
        format!("{prefix}.red"),
        format!("{:.4}", c.0 as f64 / 255.0),
    );
    vars.insert(
        format!("{prefix}.green"),
        format!("{:.4}", c.1 as f64 / 255.0),
    );
    vars.insert(
        format!("{prefix}.blue"),
        format!("{:.4}", c.2 as f64 / 255.0),
    );
}

/// build vars hashmap with different color formats for different tools
pub fn build_variables(palette: &[Rgb], wallpaper_path: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    for (i, c) in palette.iter().enumerate() {
        insert_color_vars(&mut vars, &format!("color{i}"), c);
    }

    // semantic labels derived from the palette
    let labels = colors::assign_labels(palette);
    insert_color_vars(&mut vars, "background", &labels.background);
    insert_color_vars(&mut vars, "foreground", &labels.foreground);
    insert_color_vars(&mut vars, "primary", &labels.primary);
    insert_color_vars(&mut vars, "secondary", &labels.secondary);

    // insert the wallpaper as a var for tools like lock screens
    vars.insert("wallpaper".to_string(), wallpaper_path.to_string());
    vars
}

/// Find and replace keyed values in templates.
///
/// Used {{ }} to avoid having to deal with the single brackets that
/// both css and json use.
pub fn substitute(input: &str, vars: &HashMap<String, String>) -> String {
    // with capacity reduces the amount of allocations you have to do
    // by initializing the string with a set capacity by default. useful
    // for when you are doing a ton of pushes to the string (like here)
    let mut result = String::with_capacity(input.len());
    let mut rest = input;

    // while there is text to go through, try to find {{...
    while let Some(start) = rest.find("{{") {
        // push the text up to the {{
        result.push_str(&rest[..start]);
        // store the rest of the data which comes after the {{
        let after_open = &rest[start + 2..];

        // if there is an ending }}...
        if let Some(end) = after_open.find("}}") {
            // the stuff in the middle is the key
            let key = &after_open[..end];
            // if that key exists in the valid keys hashmap...
            if let Some(val) = vars.get(key) {
                // push the val from the key into the result
                result.push_str(val);
            } else {
                // {{ = literal {. So this prints \{\{key\}\}
                eprintln!("warning: unknown template variable '{{{{{key}}}}}'");
                // keep the entire invalid key for the user to debug
                result.push_str(&rest[start..start + 2 + end + 2]);
            }
            // continue after the }}
            rest = &after_open[end + 2..];
        } else {
            // unclosed {{ — pass through literally
            result.push_str("{{");
            rest = after_open;
        }
    }
    result.push_str(rest);
    result
}

/// write out the substituted template into cache
fn render_template(
    template_path: &PathBuf,
    out_dir: &PathBuf,
    vars: &HashMap<String, String>,
) -> Result<PathBuf> {
    // get the template and read it into a string
    let input = fs::read_to_string(template_path)
        .with_context(|| format!("Failed to read template: {}", template_path.display()))?;

    // subsitute out the keys
    let rendered = substitute(&input, vars);

    // create the file in the cache based on the template name
    fs::create_dir_all(out_dir).context("Failed to create output directory")?;
    let filename = template_path.file_name().unwrap();
    let out_path = out_dir.join(filename);
    fs::write(&out_path, rendered)
        .with_context(|| format!("Failed to write rendered template: {}", out_path.display()))?;

    Ok(out_path)
}

/// run the reload command when given by the config
fn run_reload(command: &str, template_name: &str) {
    match Command::new("sh").arg("-c").arg(command).status() {
        Ok(status) if !status.success() => {
            // Command ran but errored
            eprintln!(
                "warning: reload command for '{template_name}' exited with {}",
                status
            );
        }
        Err(e) => {
            // Command wasn't able to run
            eprintln!("warning: failed to run reload for '{template_name}': {e}");
        }
        // erm... awkward!
        _ => {}
    }
}

/// render only the templates listed in config
pub fn render_templates(
    palette: &[Rgb],
    wallpaper: &Path,
    templates: &[Template],
) -> Result<()> {
    if templates.is_empty() {
        return Ok(());
    }

    let templates_dir = paths::templates_dir()?;
    let wallpaper_str = wallpaper.to_string_lossy().to_string();
    let vars = build_variables(palette, &wallpaper_str);
    let out_dir = paths::cache_dir()?;

    for cfg in templates {
        let path = templates_dir.join(&cfg.name);
        if !path.is_file() {
            eprintln!("warning: template '{}' not found in {}", cfg.name, templates_dir.display());
            continue;
        }

        match render_template(&path, &out_dir, &vars) {
            Ok(out_path) => {
                println!("Rendered template: {}", out_path.display());
                if let Some(ref cmd) = cfg.reload {
                    run_reload(cmd, &cfg.name);
                }
            }
            Err(e) => {
                eprintln!("warning: {e}");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vars() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("color0".to_string(), "#ff0000".to_string());
        vars.insert("color1".to_string(), "#00ff00".to_string());
        vars.insert("wallpaper".to_string(), "/home/user/wall.png".to_string());
        vars
    }

    #[test]
    fn substitute_vars() {
        let vars = test_vars();
        let input = "a: {{color0}}; b: {{color1}};";
        assert_eq!(substitute(input, &vars), "a: #ff0000; b: #00ff00;");
    }

    #[test]
    fn substitute_unknown_key_preserved() {
        let vars = test_vars();
        let result = substitute("x: {{nope}};", &vars);
        assert_eq!(result, "x: {{nope}};");
    }

    #[test]
    fn substitute_unclosed_brace_passthrough() {
        let vars = test_vars();
        assert_eq!(substitute("{{color0", &vars), "{{color0");
    }

    #[test]
    fn substitute_realistic_css_snippet() {
        let vars = test_vars();
        let css = r#"
            * {
                background-color: {{color0}};
                color: {{color1}};
                background-image: url("{{wallpaper}}");
            }"#;
        let rendered = substitute(css, &vars);
        assert!(rendered.contains("#ff0000"));
        assert!(rendered.contains("#00ff00"));
        assert!(rendered.contains("/home/user/wall.png"));
    }
}
