use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

use app::{render, render::PathList};
use asteroids::{Asteroid, Dispersion, Particle, Point, Size, Vector};

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

// 03

#[wasm_bindgen]
pub struct Particles(Vec<Particle>);

#[wasm_bindgen]
impl Particles {
    pub fn new() -> Self {
        let mut rng = Pcg32::seed_from_u64(1979);
        Particles(
            Dispersion::new(
                150.0,
                100.0,
                Point::new(BOUNDS.width * 0.5, BOUNDS.height * 0.5),
                Vector::zero(),
            )
            .burst(&mut rng, 24),
        )
    }

    pub fn step(&mut self, dt: f64) -> () {
        if dt <= 0.0 {
            return ();
        }
        for particle in self.0.iter_mut() {
            particle.step(dt, &BOUNDS);
        }
        self.0.retain(|particle| !particle.is_expired());
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        render::particles(&self.0, &mut list);
        list
    }
}
