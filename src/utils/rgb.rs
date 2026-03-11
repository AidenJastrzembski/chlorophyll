use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    pub fn hsl(&self) -> (f64, f64, f64) {
        // convert the [0-255] to [0-1]
        let r = self.0 as f64 / 255.0;
        let g = self.1 as f64 / 255.0;
        let b = self.2 as f64 / 255.0;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_black() {
        assert_eq!(Rgb(0, 0, 0).hex(), "#000000");
    }

    #[test]
    fn hex_white() {
        assert_eq!(Rgb(255, 255, 255).hex(), "#ffffff");
    }

    #[test]
    fn hsl_black() {
        let (h, s, l) = Rgb(0, 0, 0).hsl();
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert_eq!(l, 0.0);
    }

    #[test]
    fn hsl_white() {
        let (h, s, l) = Rgb(255, 255, 255).hsl();
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert_eq!(l, 1.0);
    }
}
