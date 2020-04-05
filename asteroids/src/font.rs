use std::collections::BTreeMap as Map;

use crate::geometry::Point;

pub type Polyline = Vec<Point>;

pub struct Font {
    height: f64,
    width: f64,
    default: Vec<Polyline>,
    glyphs: Map<char, Vec<Polyline>>,
}

impl Font {
    pub fn new(height: f64) -> Self {
        let scale = height / master::HEIGHT as f64;
        Font {
            height,
            width: (master::WIDTH as f64 * scale).ceil(),
            default: path::Data(&master::DEFAULT).to_polylines(scale),
            glyphs: master::GLYPHS
                .iter()
                .map(|(c, d)| (c.clone(), path::Data(d).to_polylines(scale)))
                .collect(),
        }
    }

    pub fn typeset_line(&self, &Point { x, y }: &Point, text: &str) -> Vec<Polyline> {
        let mut position = Point::new(x, y - self.height);
        let mut line = Vec::new();
        for c in text.chars() {
            let glyph = (self.glyphs.get(&c).unwrap_or(&self.default))
                .iter()
                .map(|polyline| polyline.iter().map(|point| point.add(&position)).collect());
            line.extend(glyph);
            position.x += self.width;
        }
        line
    }
}

mod path {
    use crate::geometry::Point;

    pub enum Command {
        M(u8, u8),
        L(u8, u8),
        C(u8, u8, u8, u8, u8, u8),
        Z,
    }

    pub struct Data<'a>(pub &'a [Command]);

    fn to_point(scale: f64, x: u8, y: u8) -> Point {
        Point::new(scale * x as f64, scale * y as f64)
    }

    impl<'a> Data<'a> {
        pub fn to_polylines(self, scale: f64) -> Vec<Vec<Point>> {
            if let Some((&Command::M(x, y), commands)) = self.0.split_first() {
                let mut polylines = Vec::new();
                let mut polyline = vec![to_point(scale, x, y)];
                for command in commands {
                    match command {
                        &Command::M(x, y) => {
                            if 1 < polyline.len() {
                                polylines.push(polyline);
                            }
                            polyline = vec![to_point(scale, x, y)];
                        }
                        &Command::L(x, y) => {
                            polyline.push(to_point(scale, x, y));
                        }
                        &Command::C(x2, y2, x3, y3, x4, y4) => {
                            if let Some(p1) = polyline.last().map(|p| p.clone()) {
                                let points = flatten_bezier(
                                    8.0 + 8.0 * scale,
                                    &p1,
                                    &to_point(scale, x2, y2),
                                    &to_point(scale, x3, y3),
                                    &to_point(scale, x4, y4),
                                );
                                polyline.extend(points);
                            }
                        }
                        &Command::Z => {
                            if let Some(point) = polyline.first().map(|p| p.clone()) {
                                polyline.push(point)
                            }
                        }
                    }
                }
                if 1 < polyline.len() {
                    polylines.push(polyline);
                }
                polylines
            } else {
                Vec::new()
            }
        }
    }

    fn flatten_bezier(
        max_error: f64,
        p1: &Point,
        p2: &Point,
        p3: &Point,
        p4: &Point,
    ) -> Vec<Point> {
        let length = p1.distance(p2) + p2.distance(p3) + p3.distance(p4);
        let n = (length / max_error).ceil() as u32;
        (1..(n + 1))
            .map(|i| bezier_point(i as f64 / n as f64, p1, p2, p3, p4))
            .collect()
    }

    fn bezier_point(t: f64, p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> Point {
        let t2 = t.powi(2);
        let t3 = t.powi(3);
        let u = 1.0 - t;
        let u2 = u.powi(2);
        let u3 = u.powi(3);
        Point::new(
            u3 * p1.x + 3.0 * u2 * t * p2.x + 3.0 * u * t2 * p3.x + t3 * p4.x,
            u3 * p1.y + 3.0 * u2 * t * p2.y + 3.0 * u * t2 * p3.y + t3 * p4.y,
        )
    }
}

mod master {
    use super::path::{Command, Command::*};

    pub const HEIGHT: u8 = 48;
    pub const WIDTH: u8 = 32;
    pub const DEFAULT: [Command; 5] = [M(2, 0), L(30, 0), L(30, 48), L(2, 48), Z];
    pub const GLYPHS: [(char, &[Command]); 5] =
        [(' ', &SPACE), ('A', &A), ('S', &S), ('T', &T), ('V', &V)];

    const SPACE: [Command; 0] = [];
    const A: [Command; 5] = [M(0, 48), L(16, 0), L(32, 48), M(6, 30), L(26, 30)];
    const S: [Command; 7] = [
        M(26, 5),
        C(24, 2, 20, 0, 16, 0),
        C(10, 0, 5, 4, 5, 10),
        C(5, 16, 10, 19, 16, 22),
        C(24, 26, 29, 29, 29, 36),
        C(29, 43, 23, 48, 16, 48),
        C(10, 48, 5, 45, 3, 40),
    ];
    const T: [Command; 4] = [M(2, 0), L(30, 0), M(16, 0), L(16, 48)];
    const V: [Command; 3] = [M(0, 0), L(16, 48), L(32, 0)];
}
