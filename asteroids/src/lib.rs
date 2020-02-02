use rand::SeedableRng;
use rand_pcg::Pcg32;

mod asteroid;
mod blast;
pub mod geometry;
mod iter;
pub mod motion;
mod particle;
mod player;
mod util;

pub use asteroid::Asteroid;
pub use blast::Blast;
use geometry::{Point, Size};
pub use particle::{Dispersion, Particle};
pub use player::{Controls, Player};

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

pub struct Game {
    // rng: Pcg32,
    pub player: Player,
    pub asteroids: Vec<Asteroid>,
    pub blasts: Vec<Blast>,
}

impl Game {
    pub fn new() -> Self {
        let mut rng = Pcg32::seed_from_u64(1979);
        Game {
            player: Player::new(Point::new(BOUNDS.width / 2.0, BOUNDS.height / 2.0)),
            asteroids: Asteroid::field(&mut rng, &BOUNDS, 24),
            blasts: Vec::new(),
            // rng,
        }
    }

    pub fn step(&mut self, dt: f64, controls: Controls) -> () {
        if dt <= 0.0 {
            return ();
        }
        self.player.step(dt, &BOUNDS, controls);
        for asteroid in self.asteroids.iter_mut() {
            asteroid.step(dt, &BOUNDS);
        }
        self.blasts.extend(self.player.fire_blast());
        for blast in self.blasts.iter_mut() {
            blast.step(dt, &BOUNDS);
        }
        self.blasts.retain(|blast| !blast.is_expired());
    }
}
