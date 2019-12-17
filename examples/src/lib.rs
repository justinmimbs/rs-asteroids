use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

use app::{render, render::PathList};
use asteroids::{Asteroid, Size};

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

// 01

#[wasm_bindgen]
pub fn asteroid_grid(seed: u32, rows: u32, cols: u32) -> PathList {
    let mut rng = Pcg32::seed_from_u64(seed as u64);
    let asteroids = Asteroid::grid(&mut rng, rows, cols);
    let mut list = PathList::new();
    render::asteroids(&asteroids, &mut list);
    list
}

// 02

#[wasm_bindgen]
pub struct AsteroidField(Vec<Asteroid>);

#[wasm_bindgen]
impl AsteroidField {
    pub fn new(count: u32) -> Self {
        let mut rng = Pcg32::seed_from_u64(1979);
        AsteroidField(Asteroid::field(&mut rng, &BOUNDS, count))
    }

    pub fn step(&mut self, dt: f64) -> () {
        if dt <= 0.0 {
            return ();
        }
        for asteroid in self.0.iter_mut() {
            asteroid.step(dt, &BOUNDS);
        }
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        render::asteroids(&self.0, &mut list);
        list
    }
}
