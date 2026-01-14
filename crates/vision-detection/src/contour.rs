pub struct Contour {
    pub points: Vec<(i32, i32)>,
    pub area: f32,
    pub perimeter: f32,
}

pub fn contours_from_mask(mask: &[u8], width: u32, height: u32) -> Vec<Contour> {
    let width = width as i32;
    let height = height as i32;

    let at = |x: i32, y: i32| (x + width * y) as usize;
    let mut done = mask.to_vec();
    let mut contours = Vec::new();

    let neighbors = [
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    for y in 0..height {
        for x in 0..width {
            if mask[at(x, y)] == 255 && x > 0 && mask[at(x - 1, y)] == 0 {
                let mut contour_points: Vec<(i32, i32)> = Vec::new();

                let start_x = x;
                let start_y = y;
                let mut curr_x = x;
                let mut curr_y = y;

                let mut dir = 4;

                loop {
                    contour_points.push((curr_x, curr_y));
                    done[at(curr_x, curr_y)] = 2;

                    let mut next_found = false;
                    let mut next_x = 0;
                    let mut next_y = 0;
                    let mut next_dir = dir;

                    for i in 0..8 {
                        let try_dir = (dir + 6 + i) % 8;
                        let (dx, dy) = neighbors[dir];

                        let check_x = curr_x + dx;
                        let check_y = curr_y + dy;

                        if check_x < 0 || check_x >= width || check_y < 0 || check_y >= height {
                            continue;
                        }

                        if mask[at(check_x, check_y)] == 255 {
                            next_x = check_x;
                            next_y = check_y;
                            next_dir = (try_dir + 4) % 8;
                            next_found = true;
                            break;
                        }
                    }
                    if !next_found {
                        break;
                    }

                    curr_x = next_x;
                    curr_y = next_y;
                    dir = next_dir;

                    if curr_x == start_x && curr_y == start_y {
                        break;
                    }
                }

                if !contour_points.is_empty() {
                    contours.push(Contour {
                        points: contour_points,
                        area: 0.0,
                        perimeter: 0.0,
                    })
                }
            }
        }
    }
    contours
}
