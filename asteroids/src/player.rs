use rand_pcg::Pcg32;
use std::f64::consts::FRAC_PI_2;

use crate::asteroid::Asteroid;
use crate::blast::Blast;
use crate::geometry;
use crate::geometry::{Point, Size, Vector};
use crate::iter::{EdgesCycleIterator, EdgesIterator};
use crate::motion;
use crate::motion::{Collide, Movement, Placement};
use crate::particle::{Dispersion, Particle};
use crate::util::Timer;

const HULL: [Point; 7] = [
    Point { x: -19.0, y: -10.0 },
    Point { x: -9.0, y: -18.0 },
    Point { x: -3.0, y: -6.0 },
    Point { x: 21.0, y: 0.0 },
    Point { x: -3.0, y: 6.0 },
    Point { x: -9.0, y: 18.0 },
    Point { x: -19.0, y: 10.0 },
];

const INTERIOR: [Point; 5] = [
    Point { x: -19.0, y: -10.0 },
    Point { x: -3.0, y: -6.0 },
    Point { x: 0.0, y: 0.0 },
    Point { x: -3.0, y: 6.0 },
    Point { x: -19.0, y: 10.0 },
];

const SPACESHIP_MASS: f64 = 300.0;

const TURNING_SPEED: f64 = 1.4; // radians / second
const THRUST_SPEED: f64 = 35.0; // px / second
const POSITION_FRICTION: f64 = 0.98;
const ROTATION_FRICTION: f64 = 0.8;

const FIRING_DELAY: f64 = 1.0 / 6.0; // seconds (6 hz)
const BLAST_SPEED: f64 = 800.0; // px / second

pub struct Controls(u32);

impl Controls {
    pub fn new(input: u32) -> Self {
        Controls(input)
    }

    pub fn left(&self) -> bool {
        self.0 & 1 != 0
    }
    pub fn right(&self) -> bool {
        self.0 & 2 != 0
    }
    pub fn thrust(&self) -> bool {
        self.0 & 4 != 0
    }
    pub fn fire(&self) -> bool {
        self.0 & 8 != 0
    }
    pub fn shield(&self) -> bool {
        self.0 & 16 != 0
    }
}

struct Spaceship {
    radius: f64,
    hull: Vec<Point>,
    interior: Vec<Point>,
    shield: Vec<Point>,
}

impl Spaceship {
    fn new(radius: f64) -> Self {
        let factor = radius / 22.0;
        Spaceship {
            radius,
            hull: HULL.iter().map(|point| point.scale(factor)).collect(),
            interior: INTERIOR.iter().map(|point| point.scale(factor)).collect(),
            shield: geometry::ngon(16, radius + 1.0),
        }
    }
}

enum Aux {
    Off,
    Firing { timer: Timer },
    Shielding { delay: Timer },
}

pub struct Impact {
    pub destroyed: bool,
    pub particles: Vec<Particle>,
}

pub struct Player {
    placement: Placement,
    movement: Movement,
    spaceship: Spaceship,
    aux: Aux,
}

impl Player {
    pub fn new(position: Point) -> Self {
        Player {
            placement: Placement {
                position,
                rotation: -FRAC_PI_2,
            },
            movement: Movement {
                velocity: Point::new(0.0, 0.0),
                angular_velocity: 0.0,
            },
            spaceship: Spaceship::new(18.0),
            aux: Aux::Off,
        }
    }

    pub fn hull(&self) -> Vec<Point> {
        self.placement.transform_points(&self.spaceship.hull)
    }

    pub fn interior(&self) -> Vec<Point> {
        self.placement.transform_points(&self.spaceship.interior)
    }

    fn is_shielding(&self) -> bool {
        match &self.aux {
            Aux::Shielding { delay } if delay.is_elapsed() => true,
            _ => false,
        }
    }

    pub fn shield(&self) -> Option<Vec<Point>> {
        if self.is_shielding() {
            Some(self.placement.transform_points(&self.spaceship.shield))
        } else {
            None
        }
    }

    pub fn step(&mut self, dt: f64, bounds: &Size, controls: Controls) -> () {
        let rotation_thrust = match (controls.left(), controls.right()) {
            (true, false) => -TURNING_SPEED * dt,
            (false, true) => TURNING_SPEED * dt,
            _ => 0.0,
        };

        let rotation = self.placement.rotation
            + (self.movement.angular_velocity * ROTATION_FRICTION * dt)
            + rotation_thrust;

        let position_thrust = if controls.thrust() {
            Vector::from_polar(THRUST_SPEED * dt, rotation)
        } else {
            Vector::new(0.0, 0.0)
        };

        let position = (self.placement.position)
            .add(&self.movement.velocity.scale(POSITION_FRICTION * dt))
            .add(&position_thrust);

        self.movement.velocity = position.sub(&self.placement.position).scale(1.0 / dt);
        self.movement.angular_velocity = (rotation - self.placement.rotation) / dt;
        self.placement.position = position;
        self.placement.rotation = rotation;
        self.placement.wrap_position(bounds);
        self.aux = if controls.shield() {
            let mut delay = if let Aux::Shielding { delay } = &self.aux {
                delay.clone()
            } else {
                Timer::new(0.0)
            };
            delay.step(dt);
            Aux::Shielding { delay }
        } else if controls.fire() {
            let mut timer = match &self.aux {
                Aux::Firing { timer } if timer.is_elapsed() => Timer::new(FIRING_DELAY),
                Aux::Firing { timer } => timer.clone(),
                _ => Timer::new(0.0),
            };
            timer.step(dt);
            Aux::Firing { timer }
        } else {
            Aux::Off
        };
    }

    pub fn fire_blast(&self) -> Option<Blast> {
        match &self.aux {
            Aux::Firing { timer } if timer.is_elapsed() => {
                let speed = self.movement.velocity.length() + BLAST_SPEED;
                let angle = self.placement.rotation;
                let position = (self.placement.position)
                    .add(&Vector::from_polar(self.spaceship.radius, angle));
                Some(Blast::new(position, speed, angle))
            }
            _ => None,
        }
    }

    pub fn interact_blast(&mut self, rng: &mut Pcg32, blast: &Blast) -> Option<Impact> {
        if let Some(impact) = blast.impact(self) {
            self.movement = self.movement.add(&Movement::from_impulse(
                &self.placement.position,
                &impact.point,
                &blast.velocity().normalize().scale(impact.speed),
            ));

            if self.is_shielding() {
                self.aux = Aux::Shielding {
                    delay: Timer::new(impact.speed * 0.002),
                };
                Some(Impact {
                    destroyed: false,
                    particles: Dispersion::new(
                        impact.point.clone(),
                        self.movement.velocity.clone(),
                        100.0,
                        50.0,
                    )
                    .burst(rng, (impact.speed / 40.0).ceil() as u32),
                })
            } else {
                // explode player
                let mut particles = Dispersion::new(
                    self.placement.position.clone(),
                    self.movement.velocity.scale(0.5),
                    150.0,
                    120.0,
                )
                .burst(rng, (impact.speed / 10.0).ceil() as u32);

                let dispersion = Dispersion::new(
                    self.placement.position.clone(),
                    self.movement.velocity.clone(),
                    impact.speed,
                    impact.speed,
                );
                particles.append(&mut dispersion.explode(rng, (self.hull().iter()).edges_cycle()));
                particles.append(&mut dispersion.explode(rng, (self.interior().iter()).edges()));

                Some(Impact {
                    destroyed: true,
                    particles,
                })
            }
        } else {
            None
        }
    }

    pub fn interact_asteroid(
        &mut self,
        _rng: &mut Pcg32,
        asteroid: &mut Asteroid,
    ) -> Option<Impact> {
        if let Some((_point, self_movement, asteroid_movement)) =
            motion::collide(self, asteroid, 0.9)
        {
            self.movement = self_movement;
            asteroid.set_movement(asteroid_movement);
            Some(Impact {
                destroyed: !self.is_shielding(),
                particles: Vec::new(),
            })
        } else {
            None
        }
    }
}

impl Collide for Player {
    fn center(&self) -> &Point {
        &self.placement.position
    }
    fn radius(&self) -> f64 {
        self.spaceship.radius
    }
    fn boundary(&self) -> Vec<Point> {
        if let Some(shield) = self.shield() {
            shield
        } else {
            self.hull()
        }
    }
    fn movement(&self) -> &Movement {
        &self.movement
    }
    fn mass(&self) -> f64 {
        SPACESHIP_MASS
    }
}
