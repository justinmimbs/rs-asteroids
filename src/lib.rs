mod asteroid;
mod geometry;
mod motion;
mod view;

use asteroid::Asteroid;
use view::PathList;

use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

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
        view::asteroids(&self.asteroids, &mut list);
        list
    }
}
