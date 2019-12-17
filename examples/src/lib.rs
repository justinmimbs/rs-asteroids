use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

use app::{render, render::PathList};
use asteroids::Asteroid;

#[wasm_bindgen]
pub fn asteroid_grid(seed: u32, rows: u32, cols: u32) -> PathList {
    let mut rng = Pcg32::seed_from_u64(seed as u64);
    let asteroids = Asteroid::grid(&mut rng, rows, cols);
    let mut list = PathList::new();
    render::asteroids(&asteroids, &mut list);
    list
}
