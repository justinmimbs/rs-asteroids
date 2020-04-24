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
use crate::util::{Interval, Timer};
use crate::Controls;

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

const NOZZLE: Point = Point { x: -19.0, y: 5.0 };

const SPACESHIP_MASS: f64 = 300.0;

const TURNING_SPEED: f64 = 0.7; // radians / second
const THRUST_SPEED: f64 = 35.0; // px / second
const POSITION_FRICTION: f64 = 0.98;
const ROTATION_FRICTION: f64 = 0.9;

const FIRING_INTERVAL: f64 = 1.0 / 6.0; // seconds (6 hz)
const BLAST_SPEED: f64 = 800.0; // px / second

const THRUSTING_INTERVAL: f64 = 1.0 / 12.0; // seconds (12 hz)
const EXHAUST_SPEED: f64 = 120.0; // px / second
const EXHAUST_MAX_AGE: f64 = 0.2; // seconds

struct Spaceship {
    radius: f64,
    hull: Vec<Point>,
    interior: Vec<Point>,
    shield: Vec<Point>,
    nozzle: Point,
}

impl Spaceship {
    fn new(radius: f64) -> Self {
        let factor = radius / 22.0;
        Spaceship {
            radius,
            hull: HULL.iter().map(|point| point.scale(factor)).collect(),
            interior: INTERIOR.iter().map(|point| point.scale(factor)).collect(),
            shield: geometry::ngon(16, radius + 1.0),
            nozzle: NOZZLE.scale(factor),
        }
    }
}

enum Aux {
    Off,
    Firing { interval: Interval },
    Shielding { delay: Timer },
}

enum Engine {
    Idle,
    Thrusting { interval: Interval },
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
    engine: Engine,
    exhaust: Vec<Timer>,
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
            engine: Engine::Idle,
            exhaust: Vec::new(),
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

    pub fn exhaust(&self) -> Vec<(f64, Vec<Point>)> {
        self.exhaust
            .iter()
            .map(|timer| {
                let alpha = timer.remaining() / EXHAUST_MAX_AGE;
                let distance = (EXHAUST_MAX_AGE - timer.remaining()) * EXHAUST_SPEED;
                let Point { x, y } = self.spaceship.nozzle;
                let path = vec![
                    Point::new(x, -y),
                    Point::new(x - distance, 0.0),
                    Point::new(x, y),
                ];
                (alpha, self.placement.transform_points(&path))
            })
            .collect()
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

        // aux
        if controls.shield() {
            if let Aux::Shielding { delay } = &mut self.aux {
                delay.step(dt);
            } else {
                self.aux = Aux::Shielding {
                    delay: Timer::new(0.0),
                };
            };
        } else if controls.fire() {
            if let Aux::Firing { interval } = &mut self.aux {
                interval.step(dt);
            } else {
                self.aux = Aux::Firing {
                    interval: Interval::new(FIRING_INTERVAL, FIRING_INTERVAL),
                };
            };
        } else {
            self.aux = Aux::Off;
        }

        // engine
        if controls.thrust() {
            if let Engine::Idle = &self.engine {
                self.engine = Engine::Thrusting {
                    interval: Interval::new(THRUSTING_INTERVAL, THRUSTING_INTERVAL),
                };
            }
        } else {
            self.engine = Engine::Idle;
        }

        // exhaust
        for timer in self.exhaust.iter_mut() {
            timer.step(dt);
        }
        if let Engine::Thrusting { interval } = &mut self.engine {
            interval.step(dt);
            self.exhaust
                .extend(interval.map(|age| Timer::new(EXHAUST_MAX_AGE - age)));
        }
        self.exhaust.retain(|timer| !timer.is_elapsed());
    }

    pub fn fire_blast(&mut self) -> Option<Blast> {
        match &mut self.aux {
            Aux::Firing { interval } => interval.next().map(|_| {
                let speed = self.movement.velocity.length() + BLAST_SPEED;
                let angle = self.placement.rotation;
                let position = (self.placement.position)
                    .add(&Vector::from_polar(self.spaceship.radius, angle));
                Blast::new(position, speed, angle)
            }),
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
            Some(self.impact(rng, &impact.point, impact.speed))
        } else {
            None
        }
    }

    pub fn interact_asteroid(
        &mut self,
        rng: &mut Pcg32,
        asteroid: &mut Asteroid,
    ) -> Option<Impact> {
        let elasticity = if self.is_shielding() { 1.0 } else { 0.1 };
        if let Some((impact_point, self_movement, asteroid_movement)) =
            motion::collide(self, asteroid, elasticity)
        {
            self.movement = self_movement;
            asteroid.set_movement(asteroid_movement);
            let impact_speed =
                self.movement.velocity.length() + asteroid.movement().velocity.length();
            Some(self.impact(rng, &impact_point, impact_speed))
        } else {
            None
        }
    }

    fn impact(&mut self, rng: &mut Pcg32, point: &Point, speed: f64) -> Impact {
        if self.is_shielding() {
            // bounce
            self.aux = Aux::Shielding {
                delay: Timer::new(speed * 0.002),
            };
            Impact {
                destroyed: false,
                particles: Dispersion::new(
                    point.clone(),
                    self.movement.velocity.scale(0.5),
                    speed * 0.5,
                    speed * 0.2,
                )
                .burst(rng, (speed.sqrt() * 0.5).ceil() as u32),
            }
        } else {
            // explode
            let mut particles = Dispersion::new(
                self.placement.position.clone(),
                self.movement.velocity.scale(0.5),
                150.0,
                120.0,
            )
            .burst(rng, speed.sqrt().ceil() as u32);

            let dispersion = Dispersion::new(
                self.placement.position.clone(),
                self.movement.velocity.clone(),
                speed,
                speed,
            );
            particles.append(&mut dispersion.explode(rng, (self.hull().iter()).edges_cycle()));
            particles.append(&mut dispersion.explode(rng, (self.interior().iter()).edges()));

            Impact {
                destroyed: true,
                particles,
            }
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
