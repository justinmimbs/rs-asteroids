use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::asteroid;
use crate::asteroid::Asteroid;
use crate::blast;
use crate::blast::Blast;
use crate::geometry::Size;
use crate::motion::Collide;
use crate::particle::Particle;
use crate::player;
use crate::player::{Controls, Player};

pub struct Level {
    rng: Pcg32,
    pub number: u8,
    pub score: u32,
    pub player: Option<Player>,
    pub asteroids: Vec<Asteroid>,
    pub blasts: Vec<Blast>,
    pub particles: Vec<Particle>,
}

impl Level {
    fn rng(number: u8) -> Pcg32 {
        Pcg32::seed_from_u64(1979 * 11 * number as u64)
    }

    pub fn new(number: u8, bounds: &Size) -> Self {
        Level {
            rng: Level::rng(number),
            number: number,
            score: 0,
            player: Some(Player::new(bounds.center())),
            asteroids: Level::asteroid_field(number, bounds),
            blasts: Vec::new(),
            particles: Vec::new(),
        }
    }

    pub fn asteroid_field(number: u8, bounds: &Size) -> Vec<Asteroid> {
        let count = 3 + 2 * number as u32;
        Asteroid::field(&mut Level::rng(number), bounds, count, 100.0)
    }

    pub fn step(&mut self, dt: f64, bounds: &Size, controls: Controls) -> () {
        if dt <= 0.0 {
            return ();
        }

        // step

        if let Some(player) = &mut self.player {
            player.step(dt, bounds, controls);
            self.blasts.extend(player.fire_blast());
        }

        for asteroid in self.asteroids.iter_mut() {
            asteroid.step(dt, bounds);
        }

        for blast in self.blasts.iter_mut() {
            blast.step(dt, bounds);
        }
        self.blasts.retain(|blast| !blast.is_expired());

        for particle in self.particles.iter_mut() {
            particle.step(dt, bounds);
        }
        self.particles.retain(|particle| !particle.is_expired());

        // interact: asteroids * blasts

        let mut asteroids = Vec::new();
        for asteroid in self.asteroids.drain(..) {
            if let Some((i, mut impact, hit)) =
                interact_asteroid_blasts(&mut self.rng, &asteroid, &self.blasts)
            {
                self.score += hit.score();
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

struct Hit {
    offset: f64,
    radius: f64,
    distance: f64,
}

impl Hit {
    fn score(&self) -> u32 {
        let offset_accuracy = 1.0 - (self.offset / self.radius);
        let radius_difficulty = 1.0
            - (self.radius - asteroid::MIN_RADIUS) / (asteroid::MAX_RADIUS - asteroid::MIN_RADIUS);
        let distance_difficulty = self.distance / blast::MAX_DISTANCE;
        let combined = offset_accuracy * 0.6 + radius_difficulty * 0.3 + distance_difficulty * 0.1;
        (combined.powi(2) * 100.0).ceil() as u32
    }
}

fn interact_asteroid_blasts(
    rng: &mut Pcg32,
    asteroid: &Asteroid,
    blasts: &Vec<Blast>,
) -> Option<(usize, asteroid::Impact, Hit)> {
    blasts.iter().enumerate().find_map(|(i, blast)| {
        asteroid.interact_blast(rng, blast).map(|impact| {
            let (a, b) = blast.endpoints();
            let hit = Hit {
                offset: asteroid.center().distance_to_line(&a, &b),
                radius: asteroid.radius(),
                distance: blast.distance_traveled(),
            };
            (i, impact, hit)
        })
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
