use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::asteroid::Asteroid;
use crate::blast::Blast;
use crate::geometry::Size;
use crate::player::{Controls, Player};

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

pub struct Level {
    // rng: Pcg32,
    pub player: Player,
    pub asteroids: Vec<Asteroid>,
    pub blasts: Vec<Blast>,
}

impl Level {
    pub fn new(n: u8) -> Self {
        let mut rng = Pcg32::seed_from_u64(1979 * 11 * n as u64);
        let count = 3 + 2 * n as u32;
        Level {
            player: Player::new(BOUNDS.center()),
            asteroids: Asteroid::field(&mut rng, &BOUNDS, count, 100.0),
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
