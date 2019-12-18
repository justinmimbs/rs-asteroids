use rand::Rng;
use rand_pcg::Pcg32;
use std::f64::consts::PI;

use crate::geometry::{Point, Radians, Size, Vector};
use crate::motion::{Movement, Placement};
use crate::util::Timer;

pub struct Particle {
    placement: Placement,
    movement: Movement,
    expiration: Timer,
    radius: f64,
}

impl Particle {
    pub fn step(&mut self, dt: f64, bounds: &Size) -> &mut Self {
        self.placement
            .apply_movement(&self.movement, dt)
            .wrap_position(bounds);
        self.expiration.step(dt);
        self
    }

    pub fn is_expired(&self) -> bool {
        self.expiration.is_elapsed()
    }

    pub fn endpoints(&self) -> (Point, Point) {
        let Placement { position, rotation } = &self.placement;
        (
            position.translate(-self.radius, *rotation),
            position.translate(self.radius, *rotation),
        )
    }
}

struct Deviation {
    scale_speed: f64,
    scale_distance: f64,
    angle: Radians,
    angular_velocity: Radians,
}

const BURST_DEVIATION: Deviation = Deviation {
    scale_speed: 0.9,
    scale_distance: 0.9,
    angle: PI,
    angular_velocity: 3.0 * PI,
};

const EXPLODE_DEVIATION: Deviation = Deviation {
    scale_speed: 0.5,
    scale_distance: 0.5,
    angle: 0.5 * PI,
    angular_velocity: PI,
};

pub struct Dispersion {
    speed: f64,
    distance: f64,
    position: Point,
    velocity: Vector,
}

impl Dispersion {
    pub fn new(speed: f64, distance: f64, position: Point, velocity: Vector) -> Self {
        Dispersion {
            speed,
            distance,
            position,
            velocity,
        }
    }

    pub fn burst(&self, rng: &mut Pcg32, count: u32) -> Vec<Particle> {
        let deviation = &BURST_DEVIATION;
        let central_angle = (2.0 * PI) / (count as f64);
        (0..count)
            .map(|i| {
                let speed =
                    self.speed + rng.gen_range(-1.0, 1.0) * deviation.scale_speed * self.speed;
                let distance = self.distance
                    + rng.gen_range(-1.0, 1.0) * deviation.scale_distance * self.distance;
                let angle = (i as f64) * central_angle + rng.gen_range(-1.0, 1.0) * deviation.angle;
                let angular_velocity = rng.gen_range(-1.0, 1.0) * deviation.angular_velocity;
                Particle {
                    placement: Placement {
                        position: self.position.clone(),
                        rotation: angle,
                    },
                    movement: Movement {
                        velocity: self.velocity.translate(speed, angle),
                        angular_velocity,
                    },
                    expiration: Timer::new(distance / speed),
                    radius: rng.gen_range(0.5, 2.5),
                }
            })
            .collect()
    }
}
