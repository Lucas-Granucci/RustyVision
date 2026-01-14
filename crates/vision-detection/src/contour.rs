use ndarray::ArrayView2;
use std::collections::VecDeque;
use std::ops::{Add, Sub};

pub struct Contour {
    pub points: Vec<Point>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

pub fn find_contours(mask: ArrayView2<u8>) -> Vec<Contour> {
    let (height, width) = mask.dim();

    let get_position_if_non_zero_pixel = |mask: ArrayView2<u8>, curr: Point| {
        let in_bounds =
            curr.x > -1 && curr.x < width as i32 && curr.y > -1 && curr.y < height as i32;

        if in_bounds && mask[(curr.y as usize, curr.x as usize)] != 0 {
            Some(curr)
        } else {
            None
        }
    };

    let mut diffs = VecDeque::from(vec![
        Point::new(-1, 0),  // w
        Point::new(-1, -1), // nw
        Point::new(0, -1),  // n
        Point::new(1, -1),  // ne
        Point::new(1, 0),   // e
        Point::new(1, 1),   // se
        Point::new(0, 1),   // s
        Point::new(-1, 1),  // sw
    ]);

    let mut contours: Vec<Contour> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if mask[(y, x)] == 0 {
                continue;
            }

            if mask[(y, x)] == 255 && x > 0 && mask[(y, x - 1)] == 0 {
                let adj = Point::new((x - 1) as i32, y as i32);
                let curr = Point::new(x as i32, y as i32);
                let mut contour_points: Vec<Point> = Vec::new();

                rotate_to_value(&mut diffs, adj - curr);

                if let Some(pos1) = diffs
                    .iter()
                    .find_map(|diff| get_position_if_non_zero_pixel(mask, *diff + curr))
                {
                    let mut pos2 = pos1;
                    let mut pos3 = curr;

                    loop {
                        contour_points.push(Point::new(pos3.x, pos3.y));
                        rotate_to_value(&mut diffs, pos2 - pos3);
                        let pos4 = diffs
                            .iter()
                            .rev()
                            .find_map(|diff| get_position_if_non_zero_pixel(mask, *diff + pos3))
                            .unwrap();

                        if pos4 == curr && pos3 == pos1 {
                            break;
                        }

                        pos2 = pos3;
                        pos3 = pos4;
                    }
                } else {
                    contour_points.push(curr);
                }
                contours.push(Contour {
                    points: contour_points,
                });
            }
        }
    }

    contours
}

fn rotate_to_value(values: &mut VecDeque<Point>, value: Point) {
    let rotate_pos = values
        .iter()
        .position(|x| x.x == value.x && x.y == value.y)
        .unwrap();
    values.rotate_left(rotate_pos);
}
