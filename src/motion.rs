use crate::geometry::{Point, Radians, Vector};

pub struct Placement {
    pub position: Point,
    pub rotation: Radians,
}

pub struct Movement {
    pub velocity: Vector,
    pub angular_velocity: Radians,
}

impl Placement {
    pub fn apply_movement(&mut self, movement: &Movement, dt: f64) -> &mut Self {
        self.position.x += movement.velocity.x * dt;
        self.position.y += movement.velocity.y * dt;
        self.rotation += movement.angular_velocity * dt;
        self
    }

    pub fn wrap_position(&mut self, width: f64, height: f64) -> &mut Self {
        self.position.x = self.position.x.rem_euclid(width);
        self.position.y = self.position.y.rem_euclid(height);
        self
    }
}
