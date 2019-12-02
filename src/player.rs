use crate::geometry;
use crate::geometry::{Matrix, Point};
use crate::motion::{Movement, Placement};

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

pub struct Spaceship {
    radius: f64,
    hull: Vec<Point>,
    interior: Vec<Point>,
    shield: Vec<Point>,
}

impl Spaceship {
    pub fn new(radius: f64) -> Self {
        let factor = radius / 22.0;
        Spaceship {
            radius,
            hull: HULL.iter().map(|point| point.scaled(factor)).collect(),
            interior: INTERIOR.iter().map(|point| point.scaled(factor)).collect(),
            shield: geometry::ngon(16, radius),
        }
    }
}

pub struct Player {
    placement: Placement,
    movement: Movement,
    spaceship: Spaceship,
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
        }
    }

    pub fn hull(&self) -> Vec<Point> {
        let matrix = Matrix::new(&self.placement.position, self.placement.rotation, 1.0);
        (self.spaceship.hull.iter())
            .map(|point| point.transformed(&matrix))
            .collect()
    }

    pub fn interior(&self) -> Vec<Point> {
        let matrix = Matrix::new(&self.placement.position, self.placement.rotation, 1.0);
        (self.spaceship.interior.iter())
            .map(|point| point.transformed(&matrix))
            .collect()
    }
}
