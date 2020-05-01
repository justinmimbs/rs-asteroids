use std::iter;

use crate::geometry::{Point, Polygon, Size, Vector};
use crate::motion::Collide;
use crate::util::Timer;

const MAX_DISTANCE: f64 = 1200.0;
const MASS: f64 = 200.0;

pub struct Blast {
    position: Point,
    velocity: Vector,
    expiration: Timer,
    dt: f64,
}

pub struct Impact {
    pub point: Point,
    pub speed: f64,
}

impl Blast {
    pub fn new(position: Point, speed: f64, angle: f64) -> Self {
        Blast {
            position,
            velocity: Vector::from_polar(speed, angle),
            expiration: Timer::new(MAX_DISTANCE / speed),
            dt: 0.0,
        }
    }

    pub fn step(&mut self, dt: f64, bounds: &Size) -> () {
        self.position
            .apply_velocity(&self.velocity, dt)
            .wrap(bounds);
        self.expiration.step(dt);
        self.dt = dt;
    }

    pub fn distance_traveled(&self) -> f64 {
        MAX_DISTANCE - self.velocity.length() * self.expiration.remaining()
    }

    pub fn is_expired(&self) -> bool {
        self.expiration.is_elapsed()
    }

    pub fn endpoints(&self) -> (Point, Point) {
        (
            self.position.clone(),
            self.position.sub(&self.velocity.scale(self.dt)),
        )
    }

    pub fn velocity(&self) -> &Vector {
        &self.velocity
    }

    pub fn impact<T>(&self, object: &T) -> Option<Impact>
    where
        T: Collide,
    {
        let (head, tail) = self.endpoints();
        if head.distance_squared(object.center()) < object.radius().powi(2) {
            let maybe_impact_point = {
                let intersections =
                    Polygon(&object.boundary()).intersections(iter::once((&head, &tail)));
                if head < tail {
                    intersections.into_iter().min()
                } else {
                    intersections.into_iter().max()
                }
            };
            if let Some(impact_point) = maybe_impact_point {
                Some(Impact {
                    point: impact_point,
                    speed: self.velocity.length() * (MASS / (MASS + object.mass())),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
