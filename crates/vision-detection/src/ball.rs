use crate::circle::Circle;
use ndarray::{Array3, ArrayView2, ArrayViewMut2, Axis};
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

    let max_votes = accumulator_matrix.iter().max().copied().unwrap_or(0);
    let threshold = (max_votes as f32 * 0.5).max(30.0) as usize;

    let mut candidates: Vec<Circle> = (0..accumulator_matrix.shape()[0])
        .into_par_iter()
        .flat_map(|r_idx| {
            let slice = accumulator_matrix.index_axis(ndarray::Axis(0), r_idx);
            let radius = radii[r_idx];

            slice
                .indexed_iter()
                .filter(|&(_, &votes)| votes > threshold)
                .map(move |((y, x), &votes)| Circle {
                    y: y as u32,
                    x: x as u32,
                    radius,
                    votes: votes as u32,
                })
                .collect::<Vec<_>>()
        })
        .collect();

    candidates.sort_by(|a, b| b.votes.cmp(&a.votes));

    for candidate in candidates {
        let mut is_duplicate = false;

        for kept_circle in &result {
            let dx = candidate.x as i32 - kept_circle.x as i32;
            let dy = candidate.y as i32 - kept_circle.y as i32;
            let distance_sq = (dx * dx + dy * dy) as f32;

            let radius_sum = (candidate.radius + kept_circle.radius) as f32;
            let overlap_thresh_sq = (radius_sum * 0.5).powi(2);

            if distance_sq < overlap_thresh_sq {
                is_duplicate = true;
                break;
            }
        }

        if !is_duplicate {
            result.push(candidate);
        }
    }

    info!("max_find_internal_ms: {}", start.elapsed().as_millis());
    result
}
