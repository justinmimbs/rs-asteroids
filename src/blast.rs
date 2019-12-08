use crate::geometry::{Point, Size, Vector};
use crate::util::Timer;

pub struct Blast {
    position: Point,
    velocity: Vector,
    expiration: Timer,
    dt: f64,
}

impl Blast {
    pub fn new(position: Point, speed: f64, angle: f64) -> Self {
        Blast {
            position,
            velocity: Vector::from_polar(speed, angle),
            expiration: Timer::new(1200.0 / speed),
            dt: 0.0,
        }
    }

    pub fn step(&mut self, dt: f64, bounds: &Size) -> &mut Self {
        self.position
            .apply_velocity(&self.velocity, dt)
            .wrap(bounds);
        self.expiration.step(dt);
        self.dt = dt;
        self
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
}
