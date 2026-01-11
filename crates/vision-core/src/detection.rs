use crate::config::DetectionConfig;
use vision_detection::ball::contours_from_mask;
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

pub fn run_ball_detection(mask_buf: &[u8], width: u32, height: u32, contour_buf: &mut Vec<u8>) {
    let contours = contours_from_mask(&mask_buf, width, height);
    for contour in contours {
        for (x, y) in contour.points {
            contour_buf[(y as u32 * width + x as u32) as usize] = 255;
        }
    }
}
