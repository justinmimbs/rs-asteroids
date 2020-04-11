use rand::SeedableRng;
use rand_pcg::Pcg32;

mod asteroid;
mod blast;
pub mod font;
pub mod geometry;
pub mod iter;
mod level;
pub mod motion;
mod particle;
mod player;
mod util;

pub use asteroid::Asteroid;
pub use blast::Blast;
use geometry::Size;
pub use level::Level;
pub use particle::{Dispersion, Particle};
pub use player::{Controls, Player};
use util::Timer;

pub struct Game {
    bounds: Size,
    state: State,
}

pub enum State {
    MainTitle {
        asteroids: Vec<Asteroid>,
    },
    LevelTitle {
        asteroids: Vec<Asteroid>,
        timer: Timer,
    },
    Playing {
        level: Level,
    },
}

use State::*;

impl Game {
    pub fn new() -> Self {
        let bounds = Size {
            width: 1200.0,
            height: 900.0,
        };

        Game {
            state: Game::main_title(&bounds),
            bounds,
        }
    }

    fn main_title(bounds: &Size) -> State {
        let mut rng = Pcg32::seed_from_u64(1979);
        MainTitle {
            asteroids: Asteroid::field(&mut rng, bounds, 12, 0.0),
        }
    }

    pub fn step(&mut self, dt: f64, controls: Controls) -> () {
        if dt <= 0.0 {
            return ();
        }
        match &mut self.state {
            MainTitle { asteroids } => {
                if controls.start() {
                    let timer = Timer::new(1.0);
                    let mut asteroids = Level::asteroid_field(1, &self.bounds);
                    asteroids_step(-timer.remaining(), &self.bounds, &mut asteroids);
                    self.state = LevelTitle { asteroids, timer };
                } else {
                    asteroids_step(dt, &self.bounds, asteroids);
                }
            }
            LevelTitle { asteroids, timer } => {
                timer.step(dt);
                if timer.is_elapsed() {
                    let mut level = Level::new(1, &self.bounds);
                    level.step(-timer.remaining(), &self.bounds, controls);
                    self.state = Playing { level }
                } else {
                    asteroids_step(dt, &self.bounds, asteroids);
                }
            }
            Playing { level } => {
                level.step(dt, &self.bounds, controls);
            }
        }
    }

    pub fn player(&self) -> &Option<Player> {
        if let Playing { level } = &self.state {
            &level.player
        } else {
            &None
        }
    }
    pub fn asteroids(&self) -> &[Asteroid] {
        match &self.state {
            MainTitle { asteroids } => &asteroids,
            LevelTitle { asteroids, .. } => &asteroids,
            Playing { level } => &level.asteroids,
        }
    }
    pub fn blasts(&self) -> &[Blast] {
        if let Playing { level } = &self.state {
            &level.blasts
        } else {
            &[]
        }
    }
    pub fn particles(&self) -> &[Particle] {
        if let Playing { level } = &self.state {
            &level.particles
        } else {
            &[]
        }
    }
}

fn asteroids_step(dt: f64, bounds: &Size, asteroids: &mut Vec<Asteroid>) {
    for asteroid in asteroids.iter_mut() {
        asteroid.step(dt, bounds);
    }
}
