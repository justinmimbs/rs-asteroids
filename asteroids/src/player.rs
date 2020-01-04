use std::f64::consts::FRAC_PI_2;

use crate::blast::Blast;
use crate::geometry;
use crate::geometry::{Point, Size, Vector};
use crate::motion::{Movement, Placement};
use crate::util::Timer;

const HULL: [Point; 7] = [
    Point { x: -10.0, y: 19.0 },
    Point { x: -18.0, y: 9.0 },
    Point { x: -6.0, y: 3.0 },
    Point { x: 0.0, y: -21.0 },
    Point { x: 6.0, y: 3.0 },
    Point { x: 18.0, y: 9.0 },
    Point { x: 10.0, y: 19.0 },
];

const INTERIOR: [Point; 5] = [
    Point { x: -10.0, y: 19.0 },
    Point { x: -6.0, y: 3.0 },
    Point { x: 0.0, y: 0.0 },
    Point { x: 6.0, y: 3.0 },
    Point { x: 10.0, y: 19.0 },
];

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
    Shielding,
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
                rotation: 0.0,
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
        self.placement.transform_path(&self.spaceship.hull)
    }

    pub fn interior(&self) -> Vec<Point> {
        self.placement.transform_path(&self.spaceship.interior)
    }

    pub fn shield(&self) -> Option<Vec<Point>> {
        if let Aux::Shielding = self.aux {
            Some(self.placement.transform_path(&self.spaceship.shield))
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
            Vector::from_polar(THRUST_SPEED * dt, rotation - FRAC_PI_2)
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
            Aux::Shielding
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
                let angle = self.placement.rotation - FRAC_PI_2;
                Some(Blast::new(self.placement.position.clone(), speed, angle))
            }
            _ => None,
        }
    }
}
