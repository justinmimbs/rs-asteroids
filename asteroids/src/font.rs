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
        Z,
    }

    pub struct Data<'a>(pub &'a [Command]);

    fn to_point(scale: f64, x: u8, y: u8) -> Point {
        Point::new(x as f64 * scale, y as f64 * scale)
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
}

mod master {
    use super::path::{Command, Command::*};

    pub const HEIGHT: u8 = 48;
    pub const WIDTH: u8 = 32;
    pub const DEFAULT: [Command; 5] = [M(2, 0), L(30, 0), L(30, 48), L(2, 48), Z];
    pub const GLYPHS: [(char, &[Command]); 3] = [(' ', &SPACE), ('A', &A), ('V', &V)];

    const SPACE: [Command; 0] = [];
    const A: [Command; 4] = [M(2, 48), L(16, 0), L(30, 48), Z];
    const V: [Command; 3] = [M(2, 0), L(16, 48), L(30, 0)];
}
