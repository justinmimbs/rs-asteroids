use std::f64::consts::FRAC_PI_2;

use crate::geometry::{Matrix, Point, Radians, Size, Vector};

impl Point {
    pub fn apply_velocity(&mut self, velocity: &Vector, dt: f64) -> &mut Self {
        self.x += velocity.x * dt;
        self.y += velocity.y * dt;
        self
    }

    pub fn wrap(&mut self, bounds: &Size) -> &mut Self {
        self.x = self.x.rem_euclid(bounds.width);
        self.y = self.y.rem_euclid(bounds.height);
        self
    }
}

pub struct Movement {
    pub velocity: Vector,
    pub angular_velocity: Radians,
}

impl Movement {
    pub fn from_impulse(center: &Point, contact: &Point, velocity: &Vector) -> Self {
        let direction = contact.direction_to(center);
        let speed = velocity.length();
        let angle = velocity.angle_to(&direction);
        let angular_speed = angle.signum() * (speed / contact.distance(center));
        let t = angle.abs() / FRAC_PI_2; // rotation alpha, within range [0, 2]
        Movement {
            velocity: direction.scale(speed * (1.0 - t)),
            angular_velocity: angular_speed * (if 1.0 < t { 2.0 - t } else { t }),
        }
    }

    pub fn add(&self, other: &Movement) -> Self {
        Movement {
            velocity: self.velocity.add(&other.velocity),
            angular_velocity: self.angular_velocity + other.angular_velocity,
        }
    }
}

pub struct Placement {
    pub position: Point,
    pub rotation: Radians,
}

impl Placement {
    pub fn apply_movement(&mut self, movement: &Movement, dt: f64) -> &mut Self {
        self.position.apply_velocity(&movement.velocity, dt);
        self.rotation += movement.angular_velocity * dt;
        self
    }

    pub fn wrap_position(&mut self, bounds: &Size) -> &mut Self {
        self.position.wrap(bounds);
        self
    }

    pub fn transform_path(&self, points: &Vec<Point>) -> Vec<Point> {
        let matrix = Matrix::new(&self.position, self.rotation, 1.0);
        (points.iter())
            .map(|point| point.transform(&matrix))
            .collect()
    }
}
