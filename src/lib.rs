use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

mod asteroid;
mod geometry;
mod motion;
mod player;
mod view;

use asteroid::Asteroid;
use geometry::{Point, Size};
use player::{Controls, Player};
use view::PathList;

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

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
            player: Player::new(Point::new(BOUNDS.width / 2.0, BOUNDS.height / 2.0)),
            asteroids: Asteroid::field(&mut rng, &BOUNDS, 24),
            rng,
        }
    }

    pub fn step(&mut self, dt: f64, input: u32) -> () {
        if 0.0 < dt {
            self.player.step(dt, &BOUNDS, Controls::new(input));
            for asteroid in self.asteroids.iter_mut() {
                asteroid.step(dt, &BOUNDS);
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
