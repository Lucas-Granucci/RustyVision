use crate::circle::Circle;
use ndarray::{s, Array3, ArrayView2, ArrayViewMut2};
use std::collections::HashMap;

fn hough_transform_set_radius_acum(
    edge_pixels: &[(usize, usize)],
    height: usize,
    width: usize,
    circle_points: &[(i32, i32)],
    accumulator_matrix: &mut ArrayViewMut2<'_, usize>,
) {
    let height_i32 = height as i32;
    let width_i32 = width as i32;

    for &(y, x) in edge_pixels {
        let y_i32 = y as i32;
        let x_i32 = x as i32;
        for &(dx, dy) in circle_points {
            let center_y = y_i32 - dy;
            let center_x = x_i32 - dx;

            if center_y >= 0 && center_y < height_i32 && center_x >= 0 && center_x < width_i32 {
                unsafe {
                    *accumulator_matrix.uget_mut([center_y as usize, center_x as usize]) += 1;
                }
            }
        }
    }
}

pub fn hough_transform(
    contour_arr: ArrayView2<u8>,
    circle_cache: &HashMap<u32, Vec<(i32, i32)>>,
) -> Vec<Circle> {
    let (height, width) = contour_arr.dim();

    let radii: Vec<u32> = {
        let mut keys: Vec<u32> = circle_cache.keys().copied().collect();
        keys.sort_unstable();
        keys
    };

    let radii_num: usize = radii.len();

    let edge_pixels: Vec<(usize, usize)> = contour_arr
        .indexed_iter()
        .filter_map(|((y, x), &val)| if val == 255 { Some((y, x)) } else { None })
        .collect();

    // Parameter space
    let mut accumulator_matrix: Array3<usize> = Array3::zeros((height, width, radii_num));

    // Voting
    for (frame, &radius) in radii.iter().enumerate() {
        let circle_points = &circle_cache[&radius];
        let mut radius_frame = accumulator_matrix.slice_mut(s![.., .., frame]);
        hough_transform_set_radius_acum(
            &edge_pixels,
            height,
            width,
            circle_points,
            &mut radius_frame,
        );
    }
    max_find(accumulator_matrix, radii)
}

fn max_find(accumulator_matrix: Array3<usize>, radii: Vec<u32>) -> Vec<Circle> {
    let mut result = Vec::<Circle>::new();
    let mut max_votes = 0;
    let mut best_params = (0, 0, 0);

    for ((y, x, r_idx), &votes) in accumulator_matrix.indexed_iter() {
        if votes > max_votes {
            max_votes = votes;
            let actual_radius = radii[r_idx];
            best_params = (y, x, actual_radius);
        }
    }

    result.push(Circle {
        y: best_params.0 as u32,
        x: best_params.1 as u32,
        radius: best_params.2 as u32,
        votes: 0,
    });
    result
}
