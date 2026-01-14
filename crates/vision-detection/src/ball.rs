use ndarray::{Array3, ArrayView2};

use crate::circle::Circle;

struct Accumulator {
    data: Array3<u32>,
    width: usize,
    height: usize,
    radius_range: usize,
}

impl Accumulator {
    fn new(img_w: u32, img_h: u32, r_min: u32, r_max: u32) -> Self {
        let width = img_w as usize;
        let height = img_h as usize;
        let radius_range = (r_max - r_min) as usize;

        Self {
            data: Array3::zeros((width, height, radius_range)),
            width,
            height,
            radius_range,
        }
    }

    fn vote(&mut self, x: usize, y: usize, r: usize) {
        if x < self.width && y < self.height && r < self.radius_range {
            self.data[[x, y, r]] += 1;
        }
    }
}

pub fn hough_circles(
    contour_arr: ArrayView2<u8>,
    r_min: u32,
    r_max: u32,
    circle_cache: &Vec<Vec<(i32, i32)>>,
) -> Vec<Circle> {
    let (height, width) = contour_arr.dim();
    let mut edges = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if contour_arr[(y, x)] == 255 {
                edges.push((x as i32, y as i32));
            }
        }
    }

    if edges.is_empty() {
        return Vec::new();
    }

    let mut accumulator = Accumulator::new(width as u32, height as u32, r_min, r_max);

    let w = width as i32;
    let h = height as i32;

    for r_idx in 0..(r_max - r_min) {
        let circle_points = &circle_cache[r_idx as usize];

        for &(edge_x, edge_y) in &edges {
            let ex = edge_x as i32;
            let ey = edge_y as i32;
            for &(dx, dy) in circle_points {
                let center_x = ex + dx;
                let center_y = ey + dy;

                if center_x >= 0 && center_x < w && center_y >= 0 && center_y < h {
                    accumulator.vote(center_x as usize, center_y as usize, r_idx as usize);
                }
            }
        }
    }
    let threshold = 50;
    let mut circles: Vec<Circle> = Vec::new();

    for r_idx in 0..accumulator.radius_range {
        for y in 0..accumulator.height {
            for x in 00..accumulator.width {
                let votes = accumulator.data[[x, y, r_idx]];
                if votes > threshold {
                    circles.push(Circle {
                        x: x as u32,
                        y: y as u32,
                        radius: r_min + r_idx as u32,
                        votes,
                    })
                }
            }
        }
    }

    circles
}
