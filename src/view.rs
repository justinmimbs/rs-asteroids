use crate::asteroid::Asteroid;
use crate::geometry::Point;
use crate::player::Player;

use wasm_bindgen::prelude::wasm_bindgen;

#[repr(C)]
pub struct Path {
    offset: usize,
    length: usize,
}

#[repr(u8)]
pub enum PathEnd {
    Open = 0,
    Closed = 1,
}

#[wasm_bindgen]
pub struct PathList {
    paths: Vec<Path>,
    alphas: Vec<f64>,
    ends: Vec<PathEnd>,
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

    pub fn push(&mut self, points: &mut Vec<Point>, alpha: f64, end: PathEnd) -> &mut Self {
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

    pub fn ends(&self) -> *const PathEnd {
        self.ends.as_ptr()
    }

    pub fn points_length(&self) -> usize {
        self.points.len()
    }

    pub fn points(&self) -> *const Point {
        self.points.as_ptr()
    }
}

//

pub fn asteroids<'a>(asteroids: &Vec<Asteroid>, list: &'a mut PathList) -> &'a mut PathList {
    for asteroid in asteroids.iter() {
        list.push(&mut asteroid.to_path(), 0.5, PathEnd::Closed);
    }

    list
}

pub fn player<'a>(player: &Player, list: &'a mut PathList) -> &'a mut PathList {
    list.push(&mut player.hull(), 1.0, PathEnd::Closed);
    list.push(&mut player.interior(), 0.7, PathEnd::Open);
    list
}
