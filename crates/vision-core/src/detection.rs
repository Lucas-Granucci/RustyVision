use std::collections::HashMap;

use ndarray::{Array2, ArrayView2, Zip};
use vision_detection::ball::hough_transform;
use vision_detection::color::{rgb_to_hsv, ColorRange};
use vision_detection::contour::find_contours;

pub fn run_color_mask(
    rgb_frame: ArrayView2<[u8; 3]>,
    mask: &mut Array2<u8>,
    color_lower: [u8; 3],
    color_upper: [u8; 3],
) {
    let range = ColorRange {
        lower: color_lower,
        upper: color_upper,
    };
    Zip::from(mask).and(rgb_frame).for_each(|m, &[r, g, b]| {
        let (h, s, v) = rgb_to_hsv(r, g, b);
        *m = if range.in_range(h, s, v) { 255 } else { 0 };
    })
}

pub fn detect_contours(
    mask: ArrayView2<u8>,
    contour_arr: &mut Array2<u8>,
    min_length: u32,
    min_area: f32,
) {
    contour_arr.fill(0);
    let (height, width) = mask.dim();
    let contours = find_contours(mask, min_length, min_area);

    for contour in &contours {
        for point in &contour.points {
            if point.x >= 0 && point.x < width as i32 && point.y >= 0 && point.y < height as i32 {
                contour_arr[(point.y as usize, point.x as usize)] = 255;
            }
        }
    }
}

pub fn detect_circles(
    contour_arr: ArrayView2<u8>,
    circle_arr: &mut Array2<u8>,
    circle_cache: &HashMap<u32, Vec<(i32, i32)>>,
    vote_thresh: u32,
) {
    circle_arr.fill(0);
    let (height, width) = contour_arr.dim();
    let circles = hough_transform(contour_arr.view(), circle_cache, vote_thresh);

    for circle in &circles {
        if let Some(circle_points) = circle_cache.get(&circle.radius) {
            for &(c_x, c_y) in circle_points {
                let x = circle.x as i32 + c_x;
                let y = circle.y as i32 + c_y;
                if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
                    circle_arr[(y as usize, x as usize)] = 255;
                }
            }
        }
    }
}
