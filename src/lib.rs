use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

mod asteroid;
mod geometry;
mod motion;
mod player;
mod view;

use asteroid::Asteroid;
use geometry::Point;
use player::Player;
use view::PathList;

const WIDTH: f64 = 1200.0;
const HEIGHT: f64 = 900.0;

#[wasm_bindgen]
pub struct App {
    rng: Pcg32,
    player: Player,
    asteroids: Vec<Asteroid>,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> Self {
        let mut rng = Pcg32::seed_from_u64(1979);
        App {
            player: Player::new(Point::new(WIDTH / 2.0, HEIGHT / 2.0)),
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
        view::player(&self.player, &mut list);
        view::asteroids(&self.asteroids, &mut list);
        list
    }
}
