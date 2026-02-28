use crate::config::Template;
use crate::theme::Theme;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// return the path to the templates dir
fn templates_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".config/chlorophyll/templates"))
}

/// return the path to the cache dir
fn output_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".cache/chlorophyll"))
}

/// TODO: extract some 'rgb' tuple for this use case
///
/// build vars hashmap with different color formats for different tools
fn build_variables(palette: &[(u8, u8, u8)], wallpaper_path: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    // enumerate gives the current index as well as the element
    for (i, &(r, g, b)) in palette.iter().enumerate() {
        let prefix = format!("color{i}");

        // Base css/gtk hex code
        vars.insert(prefix.clone(), format!("#{r:02x}{g:02x}{b:02x}"));
        // Some tools want hex but without the #
        vars.insert(format!("{prefix}.strip"), format!("{r:02x}{g:02x}{b:02x}"));
        // CSS rgb values
        vars.insert(format!("{prefix}.rgb"), format!("{r},{g},{b}"));
        // float based channels for tools like sway which want 0-1
        vars.insert(format!("{prefix}.red"), format!("{:.4}", r as f64 / 255.0));
        vars.insert(
            format!("{prefix}.green"),
            format!("{:.4}", g as f64 / 255.0),
        );
        vars.insert(format!("{prefix}.blue"), format!("{:.4}", b as f64 / 255.0));
    }

    // TODO: I assume this will require some tweaking. Once I write my own implementation
    // of color thief I can apply labels to the colors there. i.e. a 'foreground' color
    // (currently color0)
    // will probably want to be the color who is lightest without much saturation.

    // insert the wallpaper as a var for tools like lock screens
    vars.insert("wallpaper".to_string(), wallpaper_path.to_string());
    vars
}

/// Find and replace keyed values in templates.
///
/// Used {{ }} to avoid having to deal with the single brackets that
/// both css and json use.
fn substitute(input: &str, vars: &HashMap<String, String>) -> String {
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

///
pub fn render_templates(theme: &Theme, templates: &[Template]) -> Result<()> {
    // grab the templates dir
    let templates_dir = templates_dir()?;
    if !templates_dir.exists() {
        // user hasn't setup any templates, thats fine. Templates are
        // optional.
        return Ok(());
    }

    // grab all entries in the templates dir
    let entries: Vec<PathBuf> = fs::read_dir(&templates_dir)
        .context("Failed to read templates directory")?
        // filter entries by first converting from a result to an option (.ok())
        // get the path from the DirEntry
        .filter_map(|e| e.ok().map(|e| e.path()))
        // filter out symlinks + dirs etc.
        .filter(|p| p.is_file())
        // into Vec
        .collect();

    // if there was a templates folder but no actual templates
    // once again, totally fine since templates are optional
    if entries.is_empty() {
        return Ok(());
    }

    // generate the palette
    let palette = theme.palette()?;
    let wallpaper_str = theme.wallpaper.to_string_lossy().to_string();
    // generate the vars
    let vars = build_variables(&palette, &wallpaper_str);
    // get the cache dir
    let out_dir = output_dir()?;

    // for template (or more accurately, file) in templates/
    for path in &entries {
        match render_template(path, &out_dir, &vars) {
            // if the template renders correctly...
            Ok(out_path) => {
                // lil calm debug message
                println!("Rendered template: {}", out_path.display());
                // grab the file name which is then used to match against templates in
                // the config file so that we can run the reload command if given one
                let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
                // if there is a [templates] section where name = the name of the template file
                if let Some(cfg) = templates.iter().find(|c| c.name == filename) {
                    // if there is a reload command
                    if let Some(ref cmd) = cfg.reload {
                        // run the reload command
                        run_reload(cmd, &cfg.name);
                    }
                }
            }
            // uh oh! wrong decision mark!
            Err(e) => {
                eprintln!("warning: {e}");
            }
        }
    }

    Ok(())
}
