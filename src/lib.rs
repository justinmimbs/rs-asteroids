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
pub struct Path(Vec<Point>);

#[wasm_bindgen]
impl Path {
    pub fn ngon(n: u32) -> Self {
        let n = n.max(3);
        let angle = (2.0 * PI) / (n as f64);

        Path(
            (0..n)
                .map(|i| {
                    let theta = angle * (i as f64);
                    Point::new(theta.cos(), theta.sin())
                })
                .collect(),
        )
    }

    pub fn length(&self) -> usize {
        self.0.len()
    }

    pub fn points(&self) -> *const Point {
        self.0.as_ptr()
    }
}

#[repr(C)]
pub struct PathRef {
    length: usize,
    address: *const Point,
}

#[wasm_bindgen]
pub struct PathList {
    data: Vec<Path>,
    refs: Vec<PathRef>,
}

#[wasm_bindgen]
impl PathList {
    pub fn new() -> Self {
        let data = vec![Path::ngon(3), Path::ngon(5), Path::ngon(7)];
        PathList {
            refs: data
                .iter()
                .map(|path| PathRef {
                    address: path.points(),
                    length: path.length(),
                })
                .collect(),
            data,
        }
    }

    pub fn length(&self) -> usize {
        self.data.len()
    }

    pub fn paths(&self) -> *const PathRef {
        self.refs.as_ptr()
    }
}
