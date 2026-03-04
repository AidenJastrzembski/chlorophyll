use crate::utils::rgb::Rgb;
use anyhow::{Context, Result};
use color_thief::{ColorFormat, get_palette};
use image::ImageReader;
use std::path::Path;

/// Returns all 8 palette colors sorted by vibrancy score (highest first).
/// Uses HSL-based scoring: s^3 * (1 - |l - 0.5| * 2)
pub fn scored_palette(path: &Path) -> Result<Vec<Rgb>> {
    let img = ImageReader::open(path)
        .context("Failed to open image")?
        .decode()
        .context("Failed to decode image")?;

    // convert the image to a 128x128 thumbnail so that processing is much faster
    let thumb = img.thumbnail(128, 128).to_rgb8();
    let pixels = thumb.as_raw();

    // quality 1 = thorough, 10 = fast; 5 is a good balance
    // max_colors = 8 creates a color pallete with enough colors for a nice selection
    let palette = get_palette(pixels, ColorFormat::Rgb, 8, 8)
        .map_err(|e| anyhow::anyhow!("color_thief failed: {:?}", e))?;

    // Score each color by vibrancy
    // the equation is s^3 * (1 - |l - 0.5| * 2)
    //
    // s is cubed because we want to favor colors that are more saturated,
    // then we multiply by 1 - |l - 0.5| * 2 to favor colors that are closer to 0.5 lightness
    let mut scored: Vec<(f64, Rgb)> = palette
        .iter()
        .map(|color| {
            let rgb = Rgb(color.r, color.g, color.b);
            let (_, s, l) = rgb.hsl();

            let score = if !(0.15..=0.85).contains(&l) || s < 0.25 {
                // filtered colors get a negative score so they sort to the end
                -1.0 + s * 0.01
            } else {
                s.powi(3) * (1.0 - (l - 0.5).abs() * 2.0)
            };

            (score, rgb)
        })
        .collect();

    // sort descending by score
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // if every color got filtered (all scores negative), fall back to sorting by saturation
    // with a relaxed lightness filter
    if scored[0].0 < 0.0 {
        scored = palette
            .iter()
            .map(|color| {
                let rgb = Rgb(color.r, color.g, color.b);
                let (_, s, l) = rgb.hsl();
                let score = if (0.1..0.9).contains(&l) { s } else { -1.0 };
                (score, rgb)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    }

    Ok(scored.into_iter().map(|(_, color)| color).collect())
}
