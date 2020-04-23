use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::asteroid;
use crate::asteroid::Asteroid;

use crate::blast::Blast;
use crate::geometry::Size;
use crate::motion::Collide;
use crate::particle::Particle;
use crate::player;
use crate::player::{Controls, Player};

struct Stats {
    blasts_fired: u32,
    asteroids_hit: u32,
    mass_cleared: f64,
}

impl Stats {
    fn new() -> Self {
        Stats {
            blasts_fired: 0,
            asteroids_hit: 0,
            mass_cleared: 0.0,
        }
    }

    fn score(&self) -> f64 {
        let efficiency = (self.mass_cleared / self.blasts_fired as f64) / 400.0;
        let accuracy = self.asteroids_hit as f64 / self.blasts_fired as f64;
        self.mass_cleared * efficiency.sqrt() * accuracy
    }
}

pub struct Level {
    rng: Pcg32,
    pub number: u8,
    stats: Stats,
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
            stats: Stats::new(),
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

    pub fn score(&self) -> u32 {
        self.stats.score().ceil() as u32
    }

    pub fn step(&mut self, dt: f64, bounds: &Size, controls: Controls) -> () {
        if dt <= 0.0 {
            return ();
        }

        // step

        if let Some(player) = &mut self.player {
            player.step(dt, bounds, controls);
            if let Some(blast) = player.fire_blast() {
                self.stats.blasts_fired += 1;
                self.blasts.push(blast);
            }
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
            if let Some((i, mut impact)) =
                interact_asteroid_blasts(&mut self.rng, &asteroid, &self.blasts)
            {
                let remaining_mass = impact.fragments.iter().map(|f| f.mass()).sum::<f64>();
                self.stats.asteroids_hit += 1;
                self.stats.mass_cleared += asteroid.mass() - remaining_mass;
                //
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
