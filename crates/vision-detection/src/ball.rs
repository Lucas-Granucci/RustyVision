use crate::circle::Circle;
use ndarray::{s, Array3, ArrayView2, ArrayViewMut2, Axis};
use rayon::prelude::*;
use std::collections::HashMap;
use std::time::Instant;
use tracing::info;

fn hough_transform_set_radius_acum(
    edge_pixels: &[(usize, usize)],
    height: i32,
    width: i32,
    circle_points: &[(i32, i32)],
    accumulator_matrix: &mut ArrayViewMut2<'_, usize>,
) {
    let voting_start = Instant::now();
    let accum_ptr = accumulator_matrix.as_mut_ptr();
    let strides = accumulator_matrix.strides();
    let stride_row = strides[0];
    let stride_col = strides[1];

    for &(dx, dy) in circle_points {
        // dy <= y <= height + dy
        let min_y = dy.max(0) as usize;
        let max_y = (height + dy).min(height as i32) as usize;

        if min_y >= max_y {
            continue;
        }
        let start_idx = edge_pixels.partition_point(|&(y, _)| y < min_y);
        for &(y, x) in &edge_pixels[start_idx..] {
            if y >= max_y {
                break;
            }
            let center_x_i32 = (x as i32) - dx;

            if center_x_i32 >= 0 && center_x_i32 < width {
                let center_y = (y as i32) - dy;
                unsafe {
                    let offset =
                        (center_y as isize * stride_row) + (center_x_i32 as isize * stride_col);
                    *accum_ptr.offset(offset) += 1;
                }
            }
        }
    }

    // for &(y, x) in edge_pixels {
    //     let y_i32 = y as i32;
    //     let x_i32 = x as i32;
    //     for &(dx, dy) in circle_points {
    //         let center_y = y_i32 - dy;
    //         let center_x = x_i32 - dx;

    //         if center_y >= 0 && center_y < height && center_x >= 0 && center_x < width {
    //             unsafe {
    //                 *accumulator_matrix.uget_mut([center_y as usize, center_x as usize]) += 1;
    //             }
    //         }
    //     }
    // }

    info!(
        "hough_transform_accum: {}",
        voting_start.elapsed().as_millis()
    );
}

pub fn hough_transform(
    contour_arr: ArrayView2<u8>,
    circle_cache: &HashMap<u32, Vec<(i32, i32)>>,
) -> Vec<Circle> {
    let total_start = Instant::now();
    let (height, width) = contour_arr.dim();

    let radii: Vec<u32> = {
        let mut keys: Vec<u32> = circle_cache.keys().copied().collect();
        keys.sort_unstable();
        keys
    };
    let radii_num: usize = radii.len();

    let mut edge_pixels: Vec<(usize, usize)> = contour_arr
        .indexed_iter()
        .filter_map(|((y, x), &val)| if val == 255 { Some((y, x)) } else { None })
        .collect();
    edge_pixels.sort_unstable();

    // Parameter space
    let mut accumulator_matrix: Array3<usize> = Array3::zeros((radii_num, height, width));

    // Voting
    let height = height as i32;
    let width = width as i32;

    let voting_start = Instant::now();
    accumulator_matrix
        .axis_iter_mut(Axis(0))
        .into_par_iter()
        .zip(radii.par_iter())
        .for_each(|(mut radius_frame, &radius)| {
            let circle_points = &circle_cache[&radius];
            hough_transform_set_radius_acum(
                &edge_pixels,
                height,
                width,
                circle_points,
                &mut radius_frame,
            );
        });
    info!("voting_ms: {}", voting_start.elapsed().as_millis());

    let max_find_start = Instant::now();
    let circles = max_find(accumulator_matrix, radii);
    info!("max_find_ms: {}", max_find_start.elapsed().as_millis());
    info!(
        "hough_transform_total_ms: {}",
        total_start.elapsed().as_millis()
    );
    circles
}

fn max_find(accumulator_matrix: Array3<usize>, radii: Vec<u32>) -> Vec<Circle> {
    let start = Instant::now();
    let mut result = Vec::<Circle>::new();
    let mut max_votes = 0;
    let mut best_params = (0, 0, 0);

    for ((r_idx, y, x), &votes) in accumulator_matrix.indexed_iter() {
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
    info!("max_find_internal_ms: {}", start.elapsed().as_millis());
    result
}
