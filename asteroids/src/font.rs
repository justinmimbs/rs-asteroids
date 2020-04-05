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
        H(u8),
        V(u8),
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
                                    12.0 * scale,
                                    &p1,
                                    &to_point(scale, x2, y2),
                                    &to_point(scale, x3, y3),
                                    &to_point(scale, x4, y4),
                                );
                                polyline.extend(points);
                            }
                        }
                        &Command::H(x) => {
                            if let Some(mut point) = polyline.last().map(|p| p.clone()) {
                                point.x = scale * x as f64;
                                polyline.push(point)
                            }
                        }
                        &Command::V(y) => {
                            if let Some(mut point) = polyline.last().map(|p| p.clone()) {
                                point.y = scale * y as f64;
                                polyline.push(point)
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
    pub const GLYPHS: [(char, &[Command]); 37] = [
        (' ', &SPACE),
        ('0', &DIGIT_0),
        ('1', &DIGIT_1),
        ('2', &DIGIT_2),
        ('3', &DIGIT_3),
        ('4', &DIGIT_4),
        ('5', &DIGIT_5),
        ('6', &DIGIT_6),
        ('7', &DIGIT_7),
        ('8', &DIGIT_8),
        ('9', &DIGIT_9),
        ('A', &UPPERCASE_A),
        ('B', &UPPERCASE_B),
        ('C', &UPPERCASE_C),
        ('D', &UPPERCASE_D),
        ('E', &UPPERCASE_E),
        ('F', &UPPERCASE_F),
        ('G', &UPPERCASE_G),
        ('H', &UPPERCASE_H),
        ('I', &UPPERCASE_I),
        ('J', &UPPERCASE_J),
        ('K', &UPPERCASE_K),
        ('L', &UPPERCASE_L),
        ('M', &UPPERCASE_M),
        ('N', &UPPERCASE_N),
        ('O', &UPPERCASE_O),
        ('P', &UPPERCASE_P),
        ('Q', &UPPERCASE_Q),
        ('R', &UPPERCASE_R),
        ('S', &UPPERCASE_S),
        ('T', &UPPERCASE_T),
        ('U', &UPPERCASE_U),
        ('V', &UPPERCASE_V),
        ('W', &UPPERCASE_W),
        ('X', &UPPERCASE_X),
        ('Y', &UPPERCASE_Y),
        ('Z', &UPPERCASE_Z),
    ];

    const SPACE: [Command; 0] = [];
    const DIGIT_0: [Command; 7] = [
        M(16, 48),
        C(24, 48, 30, 41, 30, 33),
        V(15),
        C(30, 7, 24, 0, 16, 0),
        C(8, 0, 2, 7, 2, 15),
        V(33),
        C(2, 41, 8, 48, 16, 48),
    ];
    const DIGIT_1: [Command; 3] = [M(9, 7), L(21, 0), V(48)];
    const DIGIT_2: [Command; 6] = [
        M(29, 48),
        H(3),
        L(17, 30),
        C(24, 21, 27, 18, 27, 12),
        C(27, 6, 23, 0, 15, 0),
        C(9, 0, 5, 4, 4, 8),
    ];
    const DIGIT_3: [Command; 7] = [
        M(5, 0),
        H(27),
        L(11, 20),
        C(11, 20, 13, 20, 16, 20),
        C(24, 20, 30, 26, 30, 34),
        C(30, 42, 24, 48, 16, 48),
        C(10, 48, 5, 45, 3, 39),
    ];
    const DIGIT_4: [Command; 5] = [M(21, 0), L(2, 34), H(30), M(24, 48), V(18)];
    const DIGIT_5: [Command; 7] = [
        M(27, 0),
        H(7),
        L(4, 22),
        C(4, 22, 10, 20, 16, 20),
        C(24, 20, 30, 26, 30, 34),
        C(30, 42, 24, 48, 16, 48),
        C(10, 48, 5, 45, 3, 39),
    ];
    const DIGIT_6: [Command; 7] = [
        M(20, 0),
        L(5, 24),
        C(8, 21, 12, 20, 16, 20),
        C(24, 20, 30, 26, 30, 34),
        C(30, 42, 24, 48, 16, 48),
        C(8, 48, 2, 42, 2, 34),
        C(2, 31, 3, 27, 5, 24),
    ];
    const DIGIT_7: [Command; 3] = [M(3, 0), H(29), L(11, 48)];
    const DIGIT_8: [Command; 10] = [
        M(16, 22),
        C(23, 22, 30, 27, 30, 35),
        C(30, 43, 23, 48, 16, 48),
        C(9, 48, 2, 43, 2, 35),
        C(2, 27, 9, 22, 16, 22),
        M(16, 22),
        C(22, 22, 28, 18, 28, 11),
        C(28, 4, 22, 0, 16, 0),
        C(10, 0, 4, 4, 4, 11),
        C(4, 18, 10, 22, 16, 22),
    ];
    const DIGIT_9: [Command; 7] = [
        M(12, 48),
        L(27, 24),
        C(24, 27, 20, 28, 16, 28),
        C(8, 28, 2, 22, 2, 14),
        C(2, 6, 8, 0, 16, 0),
        C(24, 0, 30, 6, 30, 14),
        C(30, 17, 29, 21, 27, 24),
    ];
    const UPPERCASE_A: [Command; 5] = [M(0, 48), L(16, 0), L(32, 48), M(6, 30), H(26)];
    const UPPERCASE_B: [Command; 9] = [
        M(3, 22),
        H(16),
        C(24, 22, 30, 27, 30, 35),
        C(30, 43, 24, 48, 16, 48),
        H(3),
        V(0),
        H(16),
        C(22, 0, 28, 4, 28, 11),
        C(28, 18, 22, 22, 16, 22),
    ];
    const UPPERCASE_C: [Command; 5] = [
        M(29, 40),
        C(26, 45, 22, 48, 17, 48),
        C(8, 48, 1, 39, 1, 24),
        C(1, 9, 8, 0, 17, 0),
        C(22, 0, 26, 3, 29, 8),
    ];
    const UPPERCASE_D: [Command; 6] = [
        M(30, 24),
        C(30, 9, 23, 0, 14, 0),
        H(3),
        V(48),
        H(14),
        C(23, 48, 30, 39, 30, 24),
    ];
    const UPPERCASE_E: [Command; 6] = [M(29, 0), H(3), V(48), H(29), M(3, 24), H(26)];
    const UPPERCASE_F: [Command; 5] = [M(29, 0), H(3), V(48), M(3, 24), H(26)];
    const UPPERCASE_G: [Command; 7] = [
        M(29, 8),
        C(26, 3, 22, 0, 17, 0),
        C(8, 0, 1, 9, 1, 24),
        C(1, 39, 8, 48, 17, 48),
        C(22, 48, 26, 45, 29, 40),
        V(27),
        H(15),
    ];
    const UPPERCASE_H: [Command; 6] = [M(2, 0), V(48), M(30, 0), V(48), M(2, 24), H(30)];
    const UPPERCASE_I: [Command; 6] = [M(16, 0), V(48), M(9, 0), H(23), M(9, 48), H(23)];
    const UPPERCASE_J: [Command; 4] = [
        M(25, 0),
        V(34),
        C(25, 42, 20, 48, 13, 48),
        C(8, 48, 4, 45, 2, 40),
    ];
    const UPPERCASE_K: [Command; 6] = [M(2, 0), V(48), M(29, 0), L(2, 30), M(31, 48), L(11, 20)];
    const UPPERCASE_L: [Command; 3] = [M(3, 0), V(48), H(29)];
    const UPPERCASE_M: [Command; 5] = [M(0, 48), V(0), L(16, 30), L(32, 0), V(48)];
    const UPPERCASE_N: [Command; 4] = [M(30, 0), V(48), L(2, 0), V(48)];
    const UPPERCASE_O: [Command; 5] = [
        M(32, 24),
        C(32, 9, 25, 0, 16, 0),
        C(7, 0, 0, 9, 0, 24),
        C(0, 39, 7, 48, 16, 48),
        C(25, 48, 32, 39, 32, 24),
    ];
    const UPPERCASE_P: [Command; 6] = [
        M(4, 48),
        V(0),
        H(15),
        C(23, 0, 29, 5, 29, 13),
        C(29, 21, 23, 26, 15, 26),
        H(4),
    ];
    const UPPERCASE_Q: [Command; 7] = [
        M(18, 34),
        L(32, 48),
        M(16, 0),
        C(25, 0, 32, 9, 32, 24),
        C(32, 39, 25, 48, 16, 48),
        C(7, 48, 0, 39, 0, 24),
        C(0, 9, 7, 0, 16, 0),
    ];
    const UPPERCASE_R: [Command; 8] = [
        M(4, 48),
        V(0),
        H(15),
        C(23, 0, 29, 5, 29, 13),
        C(29, 21, 23, 26, 15, 26),
        H(4),
        M(15, 26),
        L(31, 48),
    ];
    const UPPERCASE_S: [Command; 7] = [
        M(26, 5),
        C(24, 2, 20, 0, 16, 0),
        C(10, 0, 5, 4, 5, 10),
        C(5, 16, 10, 19, 16, 22),
        C(24, 26, 29, 29, 29, 36),
        C(29, 43, 23, 48, 16, 48),
        C(10, 48, 5, 45, 3, 40),
    ];
    const UPPERCASE_T: [Command; 4] = [M(16, 0), V(48), M(2, 0), H(30)];
    const UPPERCASE_U: [Command; 5] = [
        M(30, 0),
        V(33),
        C(30, 41, 24, 48, 16, 48),
        C(8, 48, 2, 41, 2, 33),
        V(0),
    ];
    const UPPERCASE_V: [Command; 3] = [M(0, 0), L(16, 48), L(32, 0)];
    const UPPERCASE_W: [Command; 5] = [M(0, 0), L(4, 48), L(16, 23), L(28, 48), L(32, 0)];
    const UPPERCASE_X: [Command; 4] = [M(2, 0), L(30, 48), M(30, 0), L(2, 48)];
    const UPPERCASE_Y: [Command; 5] = [M(2, 0), L(16, 24), L(30, 0), M(16, 24), V(48)];
    const UPPERCASE_Z: [Command; 4] = [M(2, 0), H(30), L(2, 48), H(30)];
}
