use crate::config::DetectionConfig;
use vision_detection::color::{rgb_to_hsv, ColorRange};

pub fn run_color_mask_into(rgb_buf: &[u8], cfg: &DetectionConfig, mask_buf: &mut Vec<u8>) {
    let range = ColorRange {
        lower: cfg.color_lower,
        upper: cfg.color_upper,
    };
    for (dst, rgb) in mask_buf.iter_mut().zip(rgb_buf.chunks_exact(3)) {
        let (h, s, v) = rgb_to_hsv(rgb[0], rgb[1], rgb[2]);
        *dst = if range.in_range(h, s, v) { 255 } else { 0 };
    }
}
