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

enum State {
    MainTitle {
        text: Vec<Polyline>,
        asteroids: Vec<Asteroid>,
    },
    LevelIntro {
        number: u8,
        text: Vec<Polyline>,
        asteroids: Vec<Asteroid>,
        timer: Timer,
    },
    ActiveLevel {
        level: Level,
        state: LevelState,
    },
}

enum LevelState {
    Playing,
    Cleared { text: Vec<Polyline>, timer: Timer },
    Destroyed { text: Vec<Polyline>, timer: Timer },
}

use LevelState::*;
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
            &Point::new(center.x, center.y + 3.0 * font.small.height()),
            "PRESS START",
        ));
        MainTitle {
            text,
            asteroids: Asteroid::field(&mut rng, bounds, 12, 0.0),
        }
    }

    fn level_intro(number: u8, bounds: &Size, font: &FontLibrary) -> State {
        let duration = 1.5;
        let title = format!("LEVEL {}", number);
        let text = (font.medium).typeset_line(Align::Center, &bounds.center(), &title);
        let mut asteroids = Level::asteroid_field(number, &bounds);
        asteroids_step(-duration, &bounds, &mut asteroids);
        LevelIntro {
            number: number,
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
                    self.state = Game::level_intro(1, &self.bounds, &self.font);
                } else {
                    asteroids_step(dt, &self.bounds, asteroids);
                }
            }
            LevelIntro {
                number,
                asteroids,
                timer,
                ..
            } => {
                timer.step(dt);
                if timer.is_elapsed() {
                    let mut level = Level::new(*number, &self.bounds);
                    level.step(-timer.remaining(), &self.bounds, controls);
                    self.state = ActiveLevel {
                        level,
                        state: Playing,
                    }
                } else {
                    asteroids_step(dt, &self.bounds, asteroids);
                }
            }
            ActiveLevel {
                level,
                state: state @ Playing,
            } => {
                level.step(dt, &self.bounds, controls);

                if level.asteroids.is_empty() {
                    *state = Cleared {
                        text: Vec::new(),
                        timer: Timer::new(3.0),
                    };
                } else if level.player.is_none() {
                    *state = Destroyed {
                        text: Vec::new(),
                        timer: Timer::new(7.0),
                    };
                }
            }
            ActiveLevel {
                level,
                state: Cleared { text, timer },
            } => {
                timer.step(dt);

                if timer.is_elapsed() || controls.start() {
                    self.state = Game::level_intro(level.number + 1, &self.bounds, &self.font);
                } else {
                    level.step(dt, &self.bounds, controls);

                    let t = timer.remaining();
                    if t <= 2.0 && 2.0 < dt + t {
                        *text = (self.font.medium).typeset_line(
                            Align::Center,
                            &self.bounds.center(),
                            "CLEARED",
                        );
                    }
                }
            }
            ActiveLevel {
                level,
                state: Destroyed { text, timer },
            } => {
                timer.step(dt);
                if timer.is_elapsed() {
                    self.state = Game::main_title(&self.bounds, &self.font);
                } else if controls.start() {
                    self.state = Game::level_intro(level.number, &self.bounds, &self.font);
                } else {
                    level.step(dt, &self.bounds, controls);

                    let t = timer.remaining().ceil() as u8;
                    if t <= 5 && t < (dt + timer.remaining()).ceil() as u8 {
                        let center = self.bounds.center();
                        *text = (self.font.small).typeset_line(
                            Align::Center,
                            &Point::new(center.x, center.y - self.font.medium.height()),
                            "PRESS START TO CONTINUE",
                        );
                        text.extend((self.font.medium).typeset_line(
                            Align::Center,
                            &Point::new(center.x, center.y + 2.0 * self.font.small.height()),
                            &format!("{}", t),
                        ));
                    }
                }
            }
        }
    }

    pub fn player(&self) -> &Option<Player> {
        if let ActiveLevel { level, .. } = &self.state {
            &level.player
        } else {
            &None
        }
    }
    pub fn asteroids(&self) -> &[Asteroid] {
        match &self.state {
            MainTitle { asteroids, .. } => &asteroids,
            LevelIntro { asteroids, .. } => &asteroids,
            ActiveLevel { level, .. } => &level.asteroids,
        }
    }
    pub fn blasts(&self) -> &[Blast] {
        if let ActiveLevel { level, .. } = &self.state {
            &level.blasts
        } else {
            &[]
        }
    }
    pub fn particles(&self) -> &[Particle] {
        if let ActiveLevel { level, .. } = &self.state {
            &level.particles
        } else {
            &[]
        }
    }
    pub fn text(&self) -> &[Polyline] {
        match &self.state {
            MainTitle { text, .. } => &text,
            LevelIntro { text, .. } => &text,
            ActiveLevel { state, .. } => match state {
                Playing => &[],
                Cleared { text, .. } => &text,
                Destroyed { text, .. } => &text,
            },
        }
    }
}

fn asteroids_step(dt: f64, bounds: &Size, asteroids: &mut Vec<Asteroid>) {
    for asteroid in asteroids.iter_mut() {
        asteroid.step(dt, bounds);
    }
}
