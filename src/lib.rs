use std::f64::consts::PI;
use wasm_bindgen::prelude::*;

#[repr(C)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

#[wasm_bindgen]
pub struct Polygon {
    length: u32,
    points: Vec<Point>,
}

#[wasm_bindgen]
impl Polygon {
    pub fn ngon(n: u32) -> Self {
        let n = n.max(3);
        let angle = (2.0 * PI) / (n as f64);

        Polygon {
            length: n,
            points: (0..n)
                .map(|i| {
                    let theta = angle * (i as f64);
                    Point::new(theta.cos(), theta.sin())
                })
                .collect(),
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn points(&self) -> *const Point {
        self.points.as_ptr()
    }
}
