use std::f64::consts::PI;
use wasm_bindgen::prelude::*;

#[repr(C)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

#[repr(C)]
pub struct Path {
    offset: usize,
    length: usize,
}

#[repr(u8)]
pub enum End {
    Open = 0,
    Closed = 1,
}

#[wasm_bindgen]
pub struct PathList {
    paths: Vec<Path>,
    alphas: Vec<f64>,
    ends: Vec<End>,
    points: Vec<Point>,
}

impl PathList {
    pub fn new() -> Self {
        PathList {
            paths: Vec::new(),
            points: Vec::new(),
            alphas: Vec::new(),
            ends: Vec::new(),
        }
    }

    pub fn push(&mut self, points: &mut Vec<Point>, alpha: f64, end: End) -> &mut Self {
        self.paths.push(Path {
            offset: self.points.len(),
            length: points.len(),
        });
        self.alphas.push(alpha);
        self.ends.push(end);
        self.points.append(points);
        self
    }
}

#[wasm_bindgen]
impl PathList {
    pub fn length(&self) -> usize {
        self.paths.len()
    }

    pub fn paths(&self) -> *const Path {
        self.paths.as_ptr()
    }

    pub fn alphas(&self) -> *const f64 {
        self.alphas.as_ptr()
    }

    pub fn ends(&self) -> *const End {
        self.ends.as_ptr()
    }

    pub fn points_length(&self) -> usize {
        self.points.len()
    }

    pub fn points(&self) -> *const Point {
        self.points.as_ptr()
    }
}

fn ngon(n: u32) -> Vec<Point> {
    let n = n.max(3);
    let angle = (2.0 * PI) / (n as f64);

    (0..n)
        .map(|i| {
            let theta = angle * (i as f64);
            Point::new(theta.cos(), theta.sin())
        })
        .collect()
}

fn scale<'a>(factor: f64, points: &'a mut Vec<Point>) -> &'a mut Vec<Point> {
    for mut point in points.iter_mut() {
        point.x *= factor;
        point.y *= factor;
    }
    points
}

fn translate<'a>(tx: f64, ty: f64, points: &'a mut Vec<Point>) -> &'a mut Vec<Point> {
    for mut point in points.iter_mut() {
        point.x += tx;
        point.y += ty;
    }
    points
}

#[wasm_bindgen]
pub fn example() -> PathList {
    let mut list = PathList::new();
    list.push(
        translate(50.0, 50.0, scale(40.0, &mut ngon(3))),
        0.4,
        End::Closed,
    )
    .push(
        &mut translate(150.0, 50.0, scale(40.0, &mut ngon(5))),
        0.7,
        End::Closed,
    )
    .push(
        &mut translate(250.0, 50.0, scale(40.0, &mut ngon(7))),
        1.0,
        End::Open,
    );
    list
}
