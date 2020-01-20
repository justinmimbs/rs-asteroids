use std::f64::consts::FRAC_PI_2;

use crate::geometry::{Matrix, Point, Polygon, Radians, Size, Vector};
use crate::iter::EdgesCycleIterator;

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
    pub fn zero() -> Self {
        Movement {
            velocity: Vector::zero(),
            angular_velocity: 0.0,
        }
    }

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

    pub fn interpolate(&self, other: &Movement, t: f64) -> Self {
        Movement {
            velocity: self.velocity.interpolate(&other.velocity, t),
            angular_velocity: interpolate(self.angular_velocity, other.angular_velocity, t),
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

// Collide

pub trait Collide {
    fn center(&self) -> &Point;
    fn radius(&self) -> f64;
    fn boundary(&self) -> Vec<Point>;
    fn movement(&self) -> &Movement;
    fn mass(&self) -> f64;
}

pub fn collide<T, U>(a: &T, b: &U, elasticity: f64) -> Option<(Point, Movement, Movement)>
where
    T: Collide,
    U: Collide,
{
    collision_point(a, b).map(|point| collide_at_point(&point, a, b, elasticity))
}

fn collision_point<T, U>(a: &T, b: &U) -> Option<Point>
where
    T: Collide,
    U: Collide,
{
    // do circles overlap?
    if a.center().distance(b.center()) < a.radius() + b.radius() {
        // do polygons overlap?
        if let Some(point) =
            Point::mean(&Polygon(&a.boundary()).intersections(b.boundary().iter().edges_cycle()))
        {
            // are movements colliding?
            let is_collision = {
                let a_speed = a.movement().velocity.length();
                let b_speed = b.movement().velocity.length();
                let a_is_facing =
                    a.movement().velocity.angle_between(&b.center().sub(&point)) < FRAC_PI_2;
                let b_is_facing =
                    b.movement().velocity.angle_between(&a.center().sub(&point)) < FRAC_PI_2;

                a_is_facing && b_is_facing
                    || a_is_facing && b_speed < a_speed
                    || b_is_facing && a_speed < b_speed
            };

            if is_collision {
                return Some(point);
            }
        }
    }
    None
}

fn collide_at_point<T, U>(
    point: &Point,
    a: &T,
    b: &U,
    elasticity: f64,
) -> (Point, Movement, Movement)
where
    T: Collide,
    U: Collide,
{
    let inelastic_movement =
        (a.movement()).interpolate(b.movement(), b.mass() / (a.mass() + b.mass()));

    (
        point.clone(),
        inelastic_movement.interpolate(&collision_movement(point, a, b), elasticity),
        inelastic_movement.interpolate(&collision_movement(point, b, a), elasticity),
    )
}

fn collision_movement<T, U>(point: &Point, a: &T, b: &U) -> Movement
where
    T: Collide,
    U: Collide,
{
    let reflection = (a.movement().velocity).reflect(&b.center().direction_to(a.center()));
    let contact_velocity = b.movement().velocity.add(&tangential_velocity(
        &point.sub(b.center()),
        b.movement().angular_velocity,
    ));
    let impact = Movement::from_impulse(a.center(), point, &contact_velocity);
    let t = b.mass() / (a.mass() + b.mass());
    Movement {
        velocity: (a.movement().velocity)
            .interpolate(&reflection, t)
            .add(&impact.velocity.scale(2.0 * t)),
        angular_velocity: interpolate(a.movement().angular_velocity, impact.angular_velocity, t),
    }
}

fn tangential_velocity(radial: &Vector, angular_velocity: Radians) -> Vector {
    Vector::from_polar(
        radial.length() * angular_velocity,
        radial.angle() + FRAC_PI_2,
    )
}

fn interpolate(a: f64, b: f64, t: f64) -> f64 {
    (a * (1.0 - t)) + (b * t)
}
