use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::asteroid;
use crate::asteroid::Asteroid;

use crate::blast::Blast;
use crate::geometry::Size;
use crate::motion::Collide;
use crate::particle::Particle;
use crate::player;
use crate::player::Player;
use crate::Controls;

mod stats {
    pub struct Stats {
        fired: u32,
        hit: u32,
        cleared: f64,
        score: u32, // cached
    }

    impl Stats {
        pub fn new() -> Self {
            Stats {
                fired: 0,
                hit: 0,
                cleared: 0.0,
                score: 0,
            }
        }
        pub fn increment_fired(&mut self) {
            self.fired += 1;
            self.refresh_score();
        }
        pub fn increment_hit(&mut self) {
            self.hit += 1;
            self.refresh_score();
        }
        pub fn add_cleared(&mut self, mass: f64) {
            self.cleared += mass;
            self.refresh_score();
        }
        fn refresh_score(&mut self) {
            let efficiency = (self.cleared / self.fired as f64) / 400.0;
            let accuracy = self.hit as f64 / self.fired as f64;
            self.score = (self.cleared * efficiency.sqrt() * accuracy).round() as u32;
        }
        pub fn score(&self) -> u32 {
            self.score
        }
    }
}

use stats::Stats;

pub struct Level {
    rng: Pcg32,
    number: u8,
    stats: Stats,
    player: Option<Player>,
    asteroids: Vec<Asteroid>,
    blasts: Vec<Blast>,
    particles: Vec<Particle>,
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

    pub fn number(&self) -> u8 {
        self.number
    }
    pub fn score(&self) -> u32 {
        self.stats.score()
    }
    pub fn player(&self) -> &Option<Player> {
        &self.player
    }
    pub fn asteroids(&self) -> &Vec<Asteroid> {
        &self.asteroids
    }
    pub fn blasts(&self) -> &Vec<Blast> {
        &self.blasts
    }
    pub fn particles(&self) -> &Vec<Particle> {
        &self.particles
    }

    pub fn step(&mut self, dt: f64, bounds: &Size, controls: Controls) -> () {
        if dt <= 0.0 {
            return ();
        }

        // step

        if let Some(player) = &mut self.player {
            player.step(dt, bounds, controls);
            if let Some(blast) = player.fire_blast() {
                self.stats.increment_fired();
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
                self.stats.increment_hit();
                self.stats.add_cleared(asteroid.mass() - remaining_mass);
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
