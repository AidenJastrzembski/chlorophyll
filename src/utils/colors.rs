use anyhow::{Context, Result};
use image::ImageReader;

/// Extract the dominant vibrant color from an image.
///
/// 1. Thumbnails to 64×64 for speed.
/// 2. K-means (k=8) to find dominant color clusters.
/// 3. Scores clusters by size × saturation boost, filtering near-black/white.
/// 4. Returns the highest-scoring cluster centroid as (R, G, B).
pub fn dominant_color(path: &str) -> Result<(u64, u64, u64)> {
    let img = ImageReader::open(path)
        .context("Failed to open image")?
        .decode()
        .context("Failed to decode image")?;

    let thumb = img.thumbnail(64, 64).to_rgb8();

    let pixels: Vec<[f64; 3]> = thumb
        .pixels()
        .map(|p| [p.0[0] as f64, p.0[1] as f64, p.0[2] as f64])
        .collect();

    let k = 8;
    let centroids = kmeans(&pixels, k, 20);

    // Score each cluster: size * (1.0 + saturation), skip near-black/white
    let mut best_score = -1.0f64;
    let mut best_color = [128.0, 128.0, 128.0];

    for (centroid, count) in &centroids {
        let (_, s, l) = rgb_to_hsl(centroid[0], centroid[1], centroid[2]);

        // Filter near-black and near-white
        if l < 0.08 || l > 0.92 {
            continue;
        }

        let score = (*count as f64) * (1.0 + s);
        if score > best_score {
            best_score = score;
            best_color = *centroid;
        }
    }

    Ok((
        best_color[0].round() as u64,
        best_color[1].round() as u64,
        best_color[2].round() as u64,
    ))
}

/// Simple K-means clustering on RGB pixels.
/// Returns Vec of (centroid_rgb, pixel_count).
fn kmeans(pixels: &[[f64; 3]], k: usize, iterations: usize) -> Vec<([f64; 3], usize)> {
    let n = pixels.len();
    if n == 0 {
        return vec![([0.0; 3], 0); k];
    }

    // Initialize centroids by evenly sampling the pixel list
    let mut centroids: Vec<[f64; 3]> = (0..k)
        .map(|i| pixels[i * n / k])
        .collect();

    let mut assignments = vec![0usize; n];

    for _ in 0..iterations {
        // Assign each pixel to nearest centroid
        for (i, px) in pixels.iter().enumerate() {
            let mut best_dist = f64::MAX;
            let mut best_k = 0;
            for (j, c) in centroids.iter().enumerate() {
                let dist = (px[0] - c[0]).powi(2)
                    + (px[1] - c[1]).powi(2)
                    + (px[2] - c[2]).powi(2);
                if dist < best_dist {
                    best_dist = dist;
                    best_k = j;
                }
            }
            assignments[i] = best_k;
        }

        // Recompute centroids
        let mut sums = vec![[0.0f64; 3]; k];
        let mut counts = vec![0usize; k];

        for (i, px) in pixels.iter().enumerate() {
            let c = assignments[i];
            sums[c][0] += px[0];
            sums[c][1] += px[1];
            sums[c][2] += px[2];
            counts[c] += 1;
        }

        for j in 0..k {
            if counts[j] > 0 {
                centroids[j] = [
                    sums[j][0] / counts[j] as f64,
                    sums[j][1] / counts[j] as f64,
                    sums[j][2] / counts[j] as f64,
                ];
            }
        }
    }

    // Build final counts
    let mut counts = vec![0usize; k];
    for &a in &assignments {
        counts[a] += 1;
    }

    centroids.into_iter().zip(counts).collect()
}

/// Convert RGB [0..255] to HSL [0..1].
fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-10 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < 1e-10 {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < 1e-10 {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h, s, l)
}
