use rand::SeedableRng;
use rand_pcg::Pcg32;

mod asteroid;
mod blast;
pub mod geometry;
pub mod iter;
mod level;
pub mod motion;
mod particle;
mod player;
pub mod typography;
mod util;

pub use asteroid::Asteroid;
pub use blast::Blast;
use geometry::{Point, Polyline, Size};
pub use level::Level;
pub use particle::{Dispersion, Particle};
pub use player::{Controls, Player};
use typography::{Align, Font};
use util::Timer;

pub struct Game {
    bounds: Size,
    font: FontLibrary,
    state: State,
}

struct FontLibrary {
    small: Font,
    medium: Font,
    large: Font,
}

pub enum State {
    MainTitle {
        text: Vec<Polyline>,
        asteroids: Vec<Asteroid>,
    },
    LevelTitle {
        text: Vec<Polyline>,
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
        let font = FontLibrary {
            small: Font::new(32.0),
            medium: Font::new(96.0),
            large: Font::new(144.0),
        };
        Game {
            state: Game::main_title(&bounds, &font),
            font,
            bounds,
        }
    }

    fn main_title(bounds: &Size, font: &FontLibrary) -> State {
        let mut rng = Pcg32::seed_from_u64(1979);
        let center = bounds.center();
        let mut text = font.large.typeset_line(Align::Center, &center, "ASTEROIDS");
        text.extend(font.small.typeset_line(
            Align::Center,
            &Point::new(center.x, center.y + 96.0),
            "PRESS ENTER",
        ));
        MainTitle {
            text,
            asteroids: Asteroid::field(&mut rng, bounds, 12, 0.0),
        }
    }

    fn level_title(n: u8, bounds: &Size, font: &FontLibrary) -> State {
        let duration = 1.5;
        let title = format!("LEVEL {}", n);
        let text = (font.medium).typeset_line(Align::Center, &bounds.center(), &title);
        let mut asteroids = Level::asteroid_field(n, &bounds);
        asteroids_step(-duration, &bounds, &mut asteroids);
        LevelTitle {
            text,
            asteroids,
            timer: Timer::new(duration),
        }
    }

    pub fn step(&mut self, dt: f64, controls: Controls) -> () {
        if dt <= 0.0 {
            return ();
        }
        match &mut self.state {
            MainTitle { asteroids, .. } => {
                if controls.start() {
                    self.state = Game::level_title(1, &self.bounds, &self.font);
                } else {
                    asteroids_step(dt, &self.bounds, asteroids);
                }
            }
            LevelTitle {
                asteroids, timer, ..
            } => {
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
            MainTitle { asteroids, .. } => &asteroids,
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
    pub fn text(&self) -> &[Polyline] {
        match &self.state {
            MainTitle { text, .. } => &text,
            LevelTitle { text, .. } => &text,
            Playing { .. } => &[],
        }
    }
}

fn asteroids_step(dt: f64, bounds: &Size, asteroids: &mut Vec<Asteroid>) {
    for asteroid in asteroids.iter_mut() {
        asteroid.step(dt, bounds);
    }
}
