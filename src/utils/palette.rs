use crate::utils::PaletteSize;
use crate::utils::quantize;
use crate::utils::colorspace::{Hsl, Rgb};
use anyhow::{Context, Result};
use image::ImageReader;
use std::path::Path;

struct ScoredColor {
    score: f64,
    color: Rgb,
}

/// Score and sort a palette by a given scoring function, highest first.
fn score_and_sort(palette: &[Rgb], scorer: impl Fn(&Hsl) -> f64) -> Vec<ScoredColor> {
    let mut scored: Vec<ScoredColor> = palette
        .iter()
        .map(|&color| {
            let score = scorer(&color.hsl());
            ScoredColor { score, color }
        })
        .collect();
    scored.sort_by(|a, b| b.score.total_cmp(&a.score));
    scored
}

/// Returns palette colors sorted by vibrancy score (highest first).
/// Uses HSL-based scoring: s^3 * (1 - |l - 0.5| * 2)
pub fn scored_palette(path: &Path, palette_size: PaletteSize) -> Result<Vec<Rgb>> {
    let img = ImageReader::open(path)
        .context("Failed to open image")?
        .decode()
        .context("Failed to decode image")?;

    // convert the image to a 128x128 thumbnail so that processing is much faster
    let thumb = img.thumbnail(128, 128).to_rgb8();
    let pixels = thumb.as_raw();

    let palette = quantize::quantize(pixels, palette_size.get());

    // Score each color by vibrancy
    // the equation is s^3 * (1 - |l - 0.5| * 2)
    //
    // s is cubed because we want to favor colors that are more saturated,
    // then we multiply by 1 - |l - 0.5| * 2 to favor colors that are closer to 0.5 lightness
    let scored = score_and_sort(&palette, |hsl| {
        if !(0.15..=0.85).contains(&hsl.lightness) || hsl.saturation < 0.25 {
            // filtered colors get a negative score so they sort to the end
            -1.0 + hsl.saturation * 0.01
        } else {
            hsl.saturation.powi(3) * (1.0 - (hsl.lightness - 0.5).abs() * 2.0)
        }
    });

    // if every color got filtered (all scores negative), fall back to sorting by saturation
    // with a relaxed lightness filter
    let scored = if scored[0].score < 0.0 {
        score_and_sort(&palette, |hsl| {
            if (0.1..0.9).contains(&hsl.lightness) {
                hsl.saturation
            } else {
                -1.0
            }
        })
    } else {
        scored
    };

    Ok(scored.into_iter().map(|sc| sc.color).collect())
}

/// struct that contains the labeled colors
pub struct LabeledColors {
    pub background: Rgb,
    pub foreground: Rgb,
    pub primary: Rgb,
    pub secondary: Rgb,
}

impl LabeledColors {
    /// Returns the label name for a color if it matches a labeled color.
    /// Checks in priority order: primary, secondary, background, foreground.
    pub fn label_for(&self, color: &Rgb) -> Option<&'static str> {
        [
            (&self.primary, "primary"),
            (&self.secondary, "secondary"),
            (&self.background, "bg"),
            (&self.foreground, "fg"),
        ]
        .into_iter()
        .find(|(c, _)| *c == color)
        .map(|(_, label)| label)
    }
}

// assign the actual named labels to the palette
pub fn assign_labels(palette: &[Rgb]) -> LabeledColors {
    let primary = palette[0];
    let primary_h = primary.hsl().hue;

    // pick the first palette color with a sufficiently different hue from primary
    let secondary = palette
        .iter()
        .skip(1)
        .find(|c| {
            let diff = (c.hsl().hue - primary_h).abs();
            // hue wraps around at 1.0, so take the shorter arc
            let hue_dist = diff.min(1.0 - diff);
            hue_dist > 0.05
        })
        .copied()
        .unwrap_or(palette[if palette.len() > 1 { 1 } else { 0 }]);

    let bg_score = |c: &Rgb| {
        let hsl = c.hsl();
        (1.0 - hsl.lightness) * (1.0 - hsl.saturation)
    };
    let background = *palette
        .iter()
        .max_by(|a, b| bg_score(a).total_cmp(&bg_score(b)))
        .unwrap_or(&palette[0]);

    let fg_score = |c: &Rgb| {
        let hsl = c.hsl();
        hsl.lightness * (1.0 - hsl.saturation)
    };
    let foreground = *palette
        .iter()
        .max_by(|a, b| fg_score(a).total_cmp(&fg_score(b)))
        .unwrap_or(&palette[0]);

    LabeledColors {
        background,
        foreground,
        primary,
        secondary,
    }
}
