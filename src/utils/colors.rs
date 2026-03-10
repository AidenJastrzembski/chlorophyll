use crate::utils::quantize;
use crate::utils::rgb::Rgb;
use anyhow::{Context, Result};
use image::ImageReader;
use std::path::Path;

/// Returns palette colors sorted by vibrancy score (highest first).
/// Uses HSL-based scoring: s^3 * (1 - |l - 0.5| * 2)
pub fn scored_palette(path: &Path, palette_size: usize) -> Result<Vec<Rgb>> {
    let img = ImageReader::open(path)
        .context("Failed to open image")?
        .decode()
        .context("Failed to decode image")?;

    // convert the image to a 128x128 thumbnail so that processing is much faster
    let thumb = img.thumbnail(128, 128).to_rgb8();
    let pixels = thumb.as_raw();

    let palette = quantize::quantize(pixels, palette_size);

    // Score each color by vibrancy
    // the equation is s^3 * (1 - |l - 0.5| * 2)
    //
    // s is cubed because we want to favor colors that are more saturated,
    // then we multiply by 1 - |l - 0.5| * 2 to favor colors that are closer to 0.5 lightness
    let mut scored: Vec<(f64, Rgb)> = palette
        .iter()
        .map(|&rgb| {
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
            .map(|&rgb| {
                let (_, s, l) = rgb.hsl();
                let score = if (0.1..0.9).contains(&l) { s } else { -1.0 };
                (score, rgb)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    }

    Ok(scored.into_iter().map(|(_, color)| color).collect())
}

/// struct that contains the labeled colors
pub struct LabeledColors {
    pub background: Rgb,
    pub foreground: Rgb,
    pub primary: Rgb,
    pub secondary: Rgb,
}

// assign the actual named labels to the palette
pub fn assign_labels(palette: &[Rgb]) -> LabeledColors {
    let primary = palette[0];
    // TODO: make sure this is a different hue than the primary color
    let secondary = if palette.len() > 1 {
        palette[1]
    } else {
        palette[0]
    };

    // score each color for background
    let mut background = (f64::NEG_INFINITY, palette[0]);
    let mut foreground = (f64::NEG_INFINITY, palette[0]);

    // for color in palette, score it in terms of bg and fg
    for &c in palette {
        let (_, s, l) = c.hsl();
        let bg_score = (1.0 - l) * (1.0 - s);
        let fg_score = l * (1.0 - s);

        if bg_score > background.0 {
            background = (bg_score, c);
        }
        if fg_score > foreground.0 {
            foreground = (fg_score, c);
        }
    }

    // TODO: This is gross but it works for now
    LabeledColors {
        background: background.1,
        foreground: foreground.1,
        primary,
        secondary,
    }
}
