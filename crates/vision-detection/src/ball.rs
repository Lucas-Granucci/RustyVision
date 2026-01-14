struct Accumulator {
    data: Vec<u32>,
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
            data: vec![0; width * height * radius_range],
            width,
            height,
            radius_range,
        }
    }

    fn vote(&mut self, x: usize, y: usize, r: usize) {
        if x < self.width && y < self.height && r < self.radius_range {
            let idx = x + self.width * y + self.width * self.height * r;
            self.data[idx] += 1;
        }
    }

    fn get(&self, x: usize, y: usize, r: usize) -> u32 {
        let idx = x + self.width * y + self.width * self.height * r;
        self.data[idx]
    }
}

pub struct Circle {
    pub x: u32,
    pub y: u32,
    pub radius: u32,
    pub votes: u32,
}

pub fn precompute_circle_points(r_min: u32, r_max: u32) -> Vec<Vec<(i32, i32)>> {
    let mut points = Vec::with_capacity((r_max - r_min) as usize);
    for r in r_min..r_max {
        let circle_points = get_circle_points(r);
        points.push(circle_points);
    }
    points
}

pub fn get_circle_points(radius: u32) -> Vec<(i32, i32)> {
    let r = radius as f64;
    let mut points: Vec<(i32, i32)> = Vec::with_capacity(360);
    for angle_deg in 0..360 {
        let theta = (angle_deg as f64) * std::f64::consts::PI / 180.0;
        let x = (r * theta.cos()).round() as i32;
        let y = (r * theta.sin()).round() as i32;
        points.push((x, y));
    }
    points
}

pub fn hough_circles(
    cont_buf: &[u8],
    width: u32,
    height: u32,
    r_min: u32,
    r_max: u32,
) -> Vec<Circle> {
    let mut edges = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let idx = (x + width * y) as usize;
            if cont_buf[idx] == 255 {
                edges.push((x, y));
            }
        }
    }

    if edges.is_empty() {
        return Vec::new();
    }

    let mut accumulator = Accumulator::new(width, height, r_min, r_max);
    let cache = precompute_circle_points(r_min, r_max);

    let w = width as i32;
    let h = height as i32;

    for r_idx in 0..(r_max - r_min) {
        let circle_points = &cache[r_idx as usize];

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
                let votes = accumulator.get(x, y, r_idx);
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
