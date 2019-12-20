use rand::Rng;
use rand_pcg::Pcg32;
use std::f64::consts::PI;

use crate::geometry::{Point, Radians, Size, Vector};
use crate::iter::EdgesCycleIterator;
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
    direction: Radians,
    angular_velocity: Radians,
}

const BURST_DEVIATION: Deviation = Deviation {
    scale_speed: 0.9,
    scale_distance: 0.9,
    direction: PI,
    angular_velocity: 3.0 * PI,
};

const EXPLODE_DEVIATION: Deviation = Deviation {
    scale_speed: 0.5,
    scale_distance: 0.5,
    direction: 0.5 * PI,
    angular_velocity: PI,
};

pub struct Dispersion {
    position: Point,
    velocity: Vector,
    speed: f64,
    distance: f64,
}

impl Dispersion {
    pub fn new(position: Point, velocity: Vector, speed: f64, distance: f64) -> Self {
        Dispersion {
            position,
            velocity,
            speed,
            distance,
        }
    }

    fn movement(&self, rng: &mut Pcg32, deviation: &Deviation, direction: f64) -> (Movement, f64) {
        let Dispersion {
            velocity,
            speed,
            distance,
            ..
        } = &self;

        let speed = rng.gen_range(-1.0, 1.0) * deviation.scale_speed * speed + speed;
        let distance = rng.gen_range(-1.0, 1.0) * deviation.scale_distance * distance + distance;
        let direction = rng.gen_range(-1.0, 1.0) * deviation.direction + direction;
        let angular_velocity = rng.gen_range(-1.0, 1.0) * deviation.angular_velocity;
        (
            Movement {
                velocity: velocity.translate(speed, direction),
                angular_velocity,
            },
            distance / speed,
        )
    }

    pub fn burst(&self, rng: &mut Pcg32, count: u32) -> Vec<Particle> {
        let central_angle = (2.0 * PI) / (count as f64);
        (0..count)
            .map(|i| {
                let direction = (i as f64) * central_angle;
                let (movement, duration) = self.movement(rng, &BURST_DEVIATION, direction);
                Particle {
                    placement: Placement {
                        position: self.position.clone(),
                        rotation: direction,
                    },
                    movement,
                    expiration: Timer::new(duration),
                    radius: rng.gen_range(0.5, 2.5),
                }
            })
            .collect()
    }

    pub fn explode(&self, rng: &mut Pcg32, polygon: &Vec<Point>) -> Vec<Particle> {
        polygon
            .iter()
            .edges_cycle()
            .map(|(a, b)| {
                let vector = b.sub(&a);
                let midpoint = a.midpoint(&b);
                let (movement, duration) = self.movement(rng, &EXPLODE_DEVIATION, midpoint.angle());
                Particle {
                    placement: Placement {
                        position: self.position.add(&midpoint),
                        rotation: vector.angle(),
                    },
                    movement,
                    expiration: Timer::new(duration),
                    radius: 0.5 * vector.length(),
                }
            })
            .collect()
    }
}
