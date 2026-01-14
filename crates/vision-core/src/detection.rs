use ndarray::{Array2, ArrayView2, Zip};
use vision_detection::ball::hough_circles;
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

pub fn detect_contours(mask: ArrayView2<u8>, contour_arr: &mut Array2<u8>) {
    contour_arr.fill(0);
    let (height, width) = mask.dim();
    let contours = find_contours(mask);

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
    r_min: u32,
    r_max: u32,
    circle_cache: &Vec<Vec<(i32, i32)>>,
) {
    circle_arr.fill(0);
    let circles = hough_circles(contour_arr.view(), r_min, r_max, circle_cache);

    for circle in &circles {
        let circle_points = &circle_cache[(circle.radius - r_min) as usize];
        for &(c_x, c_y) in circle_points {
            let x = circle.x + c_x as u32;
            let y = circle.y + c_y as u32;
            circle_arr[(y as usize, x as usize)] = 255;
        }
    }
}
