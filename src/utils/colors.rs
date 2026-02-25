use anyhow::{Context, Result};
use image::ImageReader;

pub fn average_image_colors(path: &str) -> Result<(u64, u64, u64)> {
    println!("path: {}", path);
    let img = ImageReader::open(path)
        .context("Failed to open image")?
        .decode()
        .context("Failed to decode image")?
        .to_rgb8();

    let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);

    let count = img.width() as u64 * img.height() as u64;
    for pixel in img.pixels() {
        r += pixel.0[0] as u64;
        g += pixel.0[1] as u64;
        b += pixel.0[2] as u64;
    }
    println!("r: {}, g: {}, b: {}", r, g, b);
    println!(
        "r/count: {}, g/count: {}, b/count: {}",
        r / count,
        g / count,
        b / count
    );
    Ok((r / count, g / count, b / count))
}
