use crate::geometry::{Matrix, Point, Radians, Size, Vector};

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

    pub fn wrap_position(&mut self, bounds: &Size) -> &mut Self {
        self.position.x = self.position.x.rem_euclid(bounds.width);
        self.position.y = self.position.y.rem_euclid(bounds.height);
        self
    }

    pub fn transform_path(&self, points: &Vec<Point>) -> Vec<Point> {
        let matrix = Matrix::new(&self.position, self.rotation, 1.0);
        (points.iter())
            .map(|point| point.transform(&matrix))
            .collect()
    }
}
