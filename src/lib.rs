mod asteroid;
mod geometry;
mod motion;

use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::*;

use asteroid::Asteroid;
use geometry::Point;

#[wasm_bindgen]
pub struct App {
    rng: Pcg32,
    asteroids: Vec<Asteroid>,
}

const WIDTH: f64 = 1200.0;
const HEIGHT: f64 = 900.0;

#[wasm_bindgen]
impl App {
    pub fn new() -> Self {
        let mut rng = Pcg32::seed_from_u64(1979);
        App {
            //asteroids: Asteroid::grid(&mut rng, 6, 4),
            asteroids: Asteroid::field(&mut rng, WIDTH, HEIGHT, 24),
            rng,
        }
    }

    pub fn step(&mut self, dt: f64) -> () {
        if 0.0 < dt {
            for asteroid in self.asteroids.iter_mut() {
                asteroid.step(WIDTH, HEIGHT, dt);
            }
        }
    }

    pub fn view(&self) -> PathList {
        let mut list = PathList::new();
        for asteroid in self.asteroids.iter() {
            list.push(&mut asteroid.to_path(), 1.0, End::Closed);
        }
        list
    }
}

// view

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
