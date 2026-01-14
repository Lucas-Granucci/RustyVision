use ndarray::ArrayView2;

pub struct Contour {
    pub points: Vec<(i32, i32)>,
    pub area: f32,
    pub perimeter: f32,
}

pub fn contours_from_mask(mask: ArrayView2<u8>) -> Vec<Contour> {
    let (height, width) = mask.dim();

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
            if mask[(y, x)] == 255 && x > 0 && mask[(y, x - 1)] == 0 {
                let mut contour_points: Vec<(i32, i32)> = Vec::new();

                let start_x = x;
                let start_y = y;
                let mut curr_x = x as i32;
                let mut curr_y = y as i32;

                let mut dir = 4;

                loop {
                    contour_points.push((curr_x, curr_y));

                    let mut next_found = false;
                    let mut next_x = 0;
                    let mut next_y = 0;
                    let mut next_dir = dir;

                    for i in 0..8 {
                        let try_dir = (dir + 6 + i) % 8;
                        let (dx, dy) = neighbors[dir];

                        let check_x = curr_x + dx;
                        let check_y = curr_y + dy;

                        if check_x < 0
                            || check_x >= width as i32
                            || check_y < 0
                            || check_y >= height as i32
                        {
                            continue;
                        }

                        if mask[(check_y as usize, check_x as usize)] == 255 {
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

                    if curr_x == start_x as i32 && curr_y == start_y as i32 {
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
