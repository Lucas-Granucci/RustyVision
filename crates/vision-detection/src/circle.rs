use std::collections::HashMap;
use std::f32::consts::PI;

pub struct Circle {
    pub x: u32,
    pub y: u32,
    pub radius: u32,
    pub votes: u32,
}

pub fn precompute_circle_points(r_min: u32, r_max: u32) -> HashMap<u32, Vec<(i32, i32)>> {
    let mut points = HashMap::with_capacity((r_max - r_min) as usize);
    let radius_pixel_step: u32 = 4;
    for r in (r_min..r_max).step_by(radius_pixel_step as usize) {
        let circle_points = get_circle_points(r);
        points.insert(r, circle_points);
    }
    points
}

fn get_circle_points(radius: u32) -> Vec<(i32, i32)> {
    let r = radius as f32;
    let num_samples = (r * 2.0 * PI / 3.0).max(60.0) as usize;
    let mut points: Vec<(i32, i32)> = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let theta = (i as f32 / num_samples as f32) * 2.0 * std::f32::consts::PI;
        let x = (r * theta.cos()).round() as i32;
        let y = (r * theta.sin()).round() as i32;
        points.push((x, y));
    }
    points
}
