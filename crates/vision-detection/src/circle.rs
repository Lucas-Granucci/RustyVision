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

fn get_circle_points(radius: u32) -> Vec<(i32, i32)> {
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
