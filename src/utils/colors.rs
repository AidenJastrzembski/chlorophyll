use anyhow::{Context, Result};
use color_thief::{ColorFormat, get_palette};
use image::ImageReader;

pub fn dominant_color(path: &str) -> Result<(u64, u64, u64)> {
    // grab the image from the path
    let img = ImageReader::open(path)
        .context("Failed to open image")?
        .decode()
        .context("Failed to decode image")?;

    // convert the image to a 128x128 thumbnail so that processing is much faster
    let thumb = img.thumbnail(128, 128).to_rgb8();
    // &[u8], RGB packed
    let pixels = thumb.as_raw();

    // quality 1 = thorough, 10 = fast; 5 is a good balance
    // max_colors = 8 creates a color pallete with enough colors for a nice selection
    let palette = get_palette(pixels, ColorFormat::Rgb, 8, 8)
        .map_err(|e| anyhow::anyhow!("color_thief failed: {:?}", e))?;

    // Pick the most vibrant color from the palette
    // the equation is s^3 * (1 - |l - 0.5| * 2)
    //
    // s is cubed because we want to favor colors that are more saturated,
    // then we multiply by 1 - |l - 0.5| * 2 to favor colors that are closer to 0.5 lightness
    let mut best_score = -1.0f64;
    let mut best_color = (128u64, 128u64, 128u64);

    // for each color in the pallete, calculate its saturation and lightness,
    // then score it based on those values
    for color in &palette {
        // convert to HSL
        let (_, s, l) = rgb_to_hsl(color.r as f64, color.g as f64, color.b as f64);

        // filter out colors that are too dark or too light
        if !(0.15..=0.85).contains(&l) {
            continue;
        }
        // filter out colors that are too desaturated
        if s < 0.25 {
            continue;
        }

        // score the color
        let score = s.powi(3) * (1.0 - (l - 0.5).abs() * 2.0); // favor mid-lightness too
        // update the best score, and the best color
        if score > best_score {
            best_score = score;
            best_color = (color.r as u64, color.g as u64, color.b as u64);
        }
    }

    // Fallback: just pick most saturated if everything got filtered
    if best_score < 0.0 {
        for color in &palette {
            let (_, s, l) = rgb_to_hsl(color.r as f64, color.g as f64, color.b as f64);
            if !(0.1..0.9).contains(&l) {
                continue;
            }
            if s > best_score {
                best_score = s;
                best_color = (color.r as u64, color.g as u64, color.b as u64);
            }
        }
    }

    Ok(best_color)
}

fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // convert the [0-255] to [0-1]
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;

    // find the max and min between the colors
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    // calculate the lightness
    // lightness is the midpoint between the most saturated and least saturated colors
    let l = (max + min) / 2.0;

    // if in a grey scale return early.
    if (max - min).abs() < 1e-10 {
        return (0.0, 0.0, l);
    }

    // calculate the saturation
    // because hsl is a bicone shape, it narrows at the edges (light and dark)
    // the maximum possible diff isnt always 1, it depends on the lightness value,
    // so we need to normalize it.
    let diff = max - min; // the diff is the chroma range.
    let s = if l > 0.5 {
        diff / (2.0 - max - min)
    } else {
        diff / (max + min)
    };

    // calculate the hue
    // the hue is an angle on the color wheel, but instead of 0-360 we use 0-1
    // the logic works by determining which color is dominant, and then adjusting by the other
    // colors.
    // Red=0°  Yellow=60°  Green=120°  Cyan=180°  Blue=240°  Magenta=300°  Red=360°
    // 0       1/6         2/6         3/6        4/6        5/6           1
    // just like the unit circle minus the pi and / 2 !
    let h = if (max - r).abs() < 1e-10 {
        ((g - b) / diff + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < 1e-10 {
        ((b - r) / diff + 2.0) / 6.0
    } else {
        ((r - g) / diff + 4.0) / 6.0
    };
    (h, s, l)
}
