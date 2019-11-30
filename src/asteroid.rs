use rand::Rng;
use rand_pcg::Pcg32;
use std::f64::consts::PI;

use crate::geometry::{Matrix, Point};
use crate::motion::{Movement, Placement};

pub struct Asteroid {
    placement: Placement,
    movement: Movement,
    polygon: Vec<Point>,
}

impl Asteroid {
    pub fn new(rng: &mut Pcg32) -> Self {
        let radius: f64 = rng.gen_range(25.0, 55.0);
        Asteroid {
            placement: Placement {
                position: Point::new(0.0, 0.0),
                rotation: 0.0,
            },
            movement: Movement {
                velocity: Point::from_polar(
                    rng.gen_range(10.0, 80.0),
                    rng.gen_range(0.0, 2.0 * PI),
                ),
                angular_velocity: rng.gen_range(-1.0, 1.0),
            },
            polygon: Asteroid::shape(rng, radius),
        }
    }

    pub fn shape(rng: &mut Pcg32, radius: f64) -> Vec<Point> {
        let n: u32 = rng.gen_range((radius / 5.0).floor() as u32, (radius / 4.0).ceil() as u32);
        let angle = (2.0 * PI) / (n as f64);
        (0..n)
            .map(|i| {
                Point::from_polar(
                    radius * rng.gen_range(0.6, 1.0),
                    angle * (i as f64) + angle * rng.gen_range(0.1, 1.0),
                )
            })
            .collect()
    }

    pub fn grid(rng: &mut Pcg32, cols: u32, rows: u32) -> Vec<Asteroid> {
        let mut list = Vec::with_capacity((cols * rows) as usize);
        for row in 0..rows {
            for col in 0..cols {
                let mut asteroid = Asteroid::new(rng);
                asteroid.placement.position.x = (100 + col * 200) as f64;
                asteroid.placement.position.y = (100 + row * 200) as f64;
                list.push(asteroid);
            }
        }
        list
    }

    pub fn field(rng: &mut Pcg32, width: f64, height: f64, count: u8) -> Vec<Asteroid> {
        let mut list = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let mut asteroid = Asteroid::new(rng);
            asteroid.placement.position.x = rng.gen_range(0.0, width);
            asteroid.placement.position.y = rng.gen_range(0.0, height);
            list.push(asteroid);
        }
        list
    }

    pub fn step(&mut self, width: f64, height: f64, dt: f64) -> &mut Self {
        self.placement
            .apply_movement(&self.movement, dt)
            .wrap_position(width, height);
        self
    }

    pub fn to_path(&self) -> Vec<Point> {
        let matrix = Matrix::new(&self.placement.position, self.placement.rotation, 1.0);
        self.polygon
            .iter()
            .map(|point| point.transformed(&matrix))
            .collect()
    }
}
