use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

mod asteroid;
mod blast;
mod geometry;
mod motion;
mod player;
mod render;
mod util;

use asteroid::Asteroid;
use blast::Blast;
use geometry::{Point, Size};
use player::{Controls, Player};
use render::PathList;

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

#[wasm_bindgen]
pub struct App {
    rng: Pcg32,
    player: Player,
    asteroids: Vec<Asteroid>,
    blasts: Vec<Blast>,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> Self {
        let mut rng = Pcg32::seed_from_u64(1979);
        App {
            player: Player::new(Point::new(BOUNDS.width / 2.0, BOUNDS.height / 2.0)),
            asteroids: Asteroid::field(&mut rng, &BOUNDS, 24),
            blasts: Vec::new(),
            rng,
        }
    }

    pub fn step(&mut self, dt: f64, input: u32) -> () {
        if dt <= 0.0 {
            return ();
        }
        self.player.step(dt, &BOUNDS, Controls::new(input));
        for asteroid in self.asteroids.iter_mut() {
            asteroid.step(dt, &BOUNDS);
        }
        self.blasts.extend(self.player.fire_blast());
        for blast in self.blasts.iter_mut() {
            blast.step(dt, &BOUNDS);
        }
        self.blasts.retain(|blast| !blast.is_expired());
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        render::player(&self.player, &mut list);
        render::asteroids(&self.asteroids, &mut list);
        render::blasts(&self.blasts, &mut list);
        list
    }
}
