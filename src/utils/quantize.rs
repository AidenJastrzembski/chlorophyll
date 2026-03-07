use crate::utils::rgb::Rgb;

// NOTE: This is kind of a stupid-ish implementation of a color palette generator
// TODO: a more complex implementation in the future, perhaps one that doesnt just split the
// image into boxes by pixel count, since i can imagine a lopsided image might not get its colors
// represented by this method
// TODO: look into kmeans since that keeps getting thrown around

/// a bounding box in the pixel set.
/// the axis with the most variance is split next
struct ColorBox {
    pixels: Vec<(u8, u8, u8)>,
    /// (min, max) for the channels across pixels in this bounding box.
    r_range: (u8, u8),
    g_range: (u8, u8),
    b_range: (u8, u8),
}

impl ColorBox {
    /// create new bounding box set
    fn new(pixels: Vec<(u8, u8, u8)>) -> Self {
        // compute channel ranges up front
        let (r_range, g_range, b_range) = channel_ranges(&pixels);
        Self {
            pixels,
            r_range,
            g_range,
            b_range,
        }
    }

    /// returns the largest span (max - min) of the channels
    /// this span is then used as the priority for choosing which box to split next.
    /// higher span = higher prio
    /// this produces a color palette which better representsthe images color distro
    fn widest_span(&self) -> u8 {
        // 0 = min, 1 = max
        let r = self.r_range.1 - self.r_range.0;
        let g = self.g_range.1 - self.g_range.0;
        let b = self.b_range.1 - self.b_range.0;
        r.max(g).max(b)
    }

    /// returns 0,1,2, which correlates to r,g,b for which one has the largest
    /// range. ties are broken in r > g > b order.
    fn widest_channel(&self) -> u8 {
        let r = self.r_range.1 - self.r_range.0;
        let g = self.g_range.1 - self.g_range.0;
        let b = self.b_range.1 - self.b_range.0;
        if r >= g && r >= b {
            0
        } else if g >= b {
            1
        } else {
            2
        }
    }

    /// consumes current box, returns two boxes
    /// boxes are subsets of the original box where sub_box1 + sub_box2 = self
    /// the pixels are sorted by the dominant channel value, and then
    /// they are divided in half. This is called 'median cut', and it gives more
    /// palette slots to densely populated colors
    fn split(mut self) -> (ColorBox, ColorBox) {
        let channel = self.widest_channel();
        self.pixels.sort_by_key(|&(r, g, b)| match channel {
            0 => r,
            1 => g,
            _ => b,
        });

        let mid = self.pixels.len() / 2;
        let right = self.pixels.split_off(mid);
        (ColorBox::new(self.pixels), ColorBox::new(right))
    }

    /// collapses each box into a single pixel by averaging the color
    /// channels independantly. This tends to be fine because at the point that this
    /// is called, the colors in the box are nearly the same
    ///
    /// perhaps a more robust solution would be to pick out the color which has the
    /// highest mode within some variance
    fn average(&self) -> Rgb {
        let (mut r_sum, mut g_sum, mut b_sum) = (0u64, 0u64, 0u64);
        for &(r, g, b) in &self.pixels {
            r_sum += r as u64;
            g_sum += g as u64;
            b_sum += b as u64;
        }
        let n = self.pixels.len() as u64;
        Rgb((r_sum / n) as u8, (g_sum / n) as u8, (b_sum / n) as u8)
    }
}

/// computes the min and max values for each color channel.
fn channel_ranges(pixels: &[(u8, u8, u8)]) -> ((u8, u8), (u8, u8), (u8, u8)) {
    let (mut r_min, mut r_max) = (u8::MAX, u8::MIN);
    let (mut g_min, mut g_max) = (u8::MAX, u8::MIN);
    let (mut b_min, mut b_max) = (u8::MAX, u8::MIN);

    for &(r, g, b) in pixels {
        r_min = r_min.min(r);
        r_max = r_max.max(r);
        g_min = g_min.min(g);
        g_max = g_max.max(g);
        b_min = b_min.min(b);
        b_max = b_max.max(b);
    }

    ((r_min, r_max), (g_min, g_max), (b_min, b_max))
}

/// turns a pixel buffer (used by image libs) into a vec with at most max_colors
/// entrys, which are representitive rgb values for the image
///
/// the idea is that all pixels are placed into a single ColorBox, then on
/// each iteration, the box with the widest color span is split, which produces
/// 2 subsets of the box. this is repeated until we have max_colors subsets of
/// the original box. Then compute the average color of each subset into a representitive
pub fn quantize(pixels: &[u8], max_colors: usize) -> Vec<Rgb> {
    let tuples: Vec<(u8, u8, u8)> = pixels.chunks_exact(3).map(|c| (c[0], c[1], c[2])).collect();

    if tuples.is_empty() || max_colors == 0 {
        return Vec::new();
    }

    let mut boxes: Vec<ColorBox> = vec![ColorBox::new(tuples)];

    while boxes.len() < max_colors {
        // pick the box with the widest channel span.
        let idx = boxes
            // create iterator
            .iter()
            // also get the index of the iterator
            .enumerate()
            // of the boxes, select the one with the widest span
            .max_by_key(|(_, b)| b.widest_span())
            // return the index of that box
            .map(|(i, _)| i)
            .unwrap();

        // take the box out of the set of boxes
        let widest = boxes.swap_remove(idx);
        // split the box
        let (left, right) = widest.split();
        // add the two new boxes to the set of boxes
        boxes.push(left);
        boxes.push(right);
    }

    // collapse each box into its average color and collect results into Vec.
    boxes.iter().map(ColorBox::average).collect()
}
