use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::asteroid;
use crate::asteroid::Asteroid;
use crate::blast::Blast;
use crate::geometry::Size;
use crate::particle::Particle;
use crate::player;
use crate::player::{Controls, Player};

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

pub struct Level {
    rng: Pcg32,
    pub player: Option<Player>,
    pub asteroids: Vec<Asteroid>,
    pub blasts: Vec<Blast>,
    pub particles: Vec<Particle>,
}

impl Level {
    pub fn new(n: u8) -> Self {
        let mut rng = Pcg32::seed_from_u64(1979 * 11 * n as u64);
        let count = 3 + 2 * n as u32;
        Level {
            player: Some(Player::new(BOUNDS.center())),
            asteroids: Asteroid::field(&mut rng, &BOUNDS, count, 100.0),
            blasts: Vec::new(),
            particles: Vec::new(),
            rng,
        }
    }

    pub fn step(&mut self, dt: f64, controls: Controls) -> () {
        if dt <= 0.0 {
            return ();
        }

        // step

        if let Some(player) = &mut self.player {
            player.step(dt, &BOUNDS, controls);
            self.blasts.extend(player.fire_blast());
        }

        for asteroid in self.asteroids.iter_mut() {
            asteroid.step(dt, &BOUNDS);
        }

        for blast in self.blasts.iter_mut() {
            blast.step(dt, &BOUNDS);
        }
        self.blasts.retain(|blast| !blast.is_expired());

        for particle in self.particles.iter_mut() {
            particle.step(dt, &BOUNDS);
        }
        self.particles.retain(|particle| !particle.is_expired());

        // interact: asteroids * blasts

        let mut asteroids = Vec::new();
        for asteroid in self.asteroids.drain(..) {
            if let Some((i, mut impact)) =
                interact_asteroid_blasts(&mut self.rng, &asteroid, &self.blasts)
            {
                self.blasts.remove(i);
                asteroids.append(&mut impact.fragments);
                self.particles.append(&mut impact.particles);
            } else {
                asteroids.push(asteroid);
            }
        }
        self.asteroids = asteroids;

        // interact: player * blasts

        if let Some(player) = &mut self.player {
            if let Some((i, mut impact)) =
                interact_player_blasts(&mut self.rng, player, &self.blasts)
            {
                self.blasts.remove(i);
                self.particles.append(&mut impact.particles);
                if impact.destroyed {
                    self.player = None;
                }
            }
        }

        // interact: player * asteroids

        if let Some(player) = &mut self.player {
            if let Some(mut impact) =
                interact_player_asteroids(&mut self.rng, player, &mut self.asteroids)
            {
                self.particles.append(&mut impact.particles);
                if impact.destroyed {
                    self.player = None;
                }
            }
        }
    }
}

fn interact_asteroid_blasts(
    rng: &mut Pcg32,
    asteroid: &Asteroid,
    blasts: &Vec<Blast>,
) -> Option<(usize, asteroid::Impact)> {
    blasts.iter().enumerate().find_map(|(i, blast)| {
        asteroid
            .interact_blast(rng, blast)
            .map(|impact| (i, impact))
    })
}

fn interact_player_blasts(
    rng: &mut Pcg32,
    player: &mut Player,
    blasts: &Vec<Blast>,
) -> Option<(usize, player::Impact)> {
    (blasts.iter().enumerate())
        .find_map(|(i, blast)| player.interact_blast(rng, blast).map(|impact| (i, impact)))
}

fn interact_player_asteroids(
    rng: &mut Pcg32,
    player: &mut Player,
    asteroids: &mut Vec<Asteroid>,
) -> Option<player::Impact> {
    (asteroids.iter_mut()).find_map(|asteroid| player.interact_asteroid(rng, asteroid))
}
