use core::f64::consts::PI;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

use app::{
    render,
    render::{PathEnd, PathList},
};
use asteroids::{geometry, geometry::Circle, Asteroid, Dispersion, Particle, Point, Size, Vector};

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

// 01

#[wasm_bindgen]
pub fn asteroid_grid(seed: u32, rows: u32, cols: u32) -> PathList {
    let mut rng = Pcg32::seed_from_u64(seed as u64);
    let asteroids = Asteroid::grid(&mut rng, rows, cols);
    let mut list = PathList::new();
    render::asteroids(&asteroids, &mut list);
    list
}

// 02

#[wasm_bindgen]
pub struct AsteroidField(Vec<Asteroid>);

#[wasm_bindgen]
impl AsteroidField {
    pub fn new(count: u32) -> Self {
        let mut rng = Pcg32::seed_from_u64(1979);
        AsteroidField(Asteroid::field(&mut rng, &BOUNDS, count))
    }

    pub fn step(&mut self, dt: f64) -> () {
        if dt <= 0.0 {
            return ();
        }
        for asteroid in self.0.iter_mut() {
            asteroid.step(dt, &BOUNDS);
        }
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        render::asteroids(&self.0, &mut list);
        list
    }
}

// 03

#[wasm_bindgen]
pub struct Particles {
    rng: Pcg32,
    target: Option<(Point, Vector)>,
    particles: Vec<Particle>,
}

#[wasm_bindgen]
impl Particles {
    pub fn new() -> Self {
        Particles {
            rng: Pcg32::seed_from_u64(1979),
            target: None,
            particles: Vec::new(),
        }
    }

    fn target_shape() -> Vec<Point> {
        geometry::ngon(5, 12.0)
    }

    pub fn ready(&mut self, position_x: f64, position_y: f64) -> () {
        self.target = Some((Point::new(position_x, position_y), Vector::zero()));
    }

    pub fn aim(&mut self, velocity_x: f64, velocity_y: f64) -> () {
        if let Some(position_velocity) = &mut self.target {
            position_velocity.1 = Vector::new(velocity_x, velocity_y);
        }
    }

    pub fn fire(&mut self) -> () {
        if let Some((position, velocity)) = &self.target {
            let mut particles = Dispersion::new(position.clone(), velocity.clone(), 150.0, 100.0)
                .burst(&mut self.rng, 24);
            self.particles.append(&mut particles);

            let mut pieces = Dispersion::new(position.clone(), velocity.clone(), 100.0, 150.0)
                .explode(&mut self.rng, &Particles::target_shape());
            self.particles.append(&mut pieces);

            self.target = None;
        }
    }

    pub fn step(&mut self, dt: f64) -> () {
        if dt <= 0.0 {
            return ();
        }
        for particle in self.particles.iter_mut() {
            particle.step(dt, &BOUNDS);
        }
        self.particles.retain(|particle| !particle.is_expired());
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        render::particles(&self.particles, &mut list);
        if let Some((position, velocity)) = &self.target {
            let mut shape = Particles::target_shape()
                .iter()
                .map(|point| point.add(position))
                .collect();
            list.push(&mut shape, 1.0, PathEnd::Closed);

            let mut line = vec![position.clone(), position.sub(velocity)];
            list.push(&mut line, 0.2, PathEnd::Open);
        }
        list
    }
}

// 04

#[wasm_bindgen]
pub struct EnclosingCircle {
    points: Vec<Point>,
}

#[wasm_bindgen]
impl EnclosingCircle {
    pub fn new() -> Self {
        EnclosingCircle { points: Vec::new() }
    }

    pub fn add(&mut self, x: f64, y: f64) -> () {
        self.points.push(Point::new(x, y));
    }

    pub fn render(&self) -> PathList {
        let initial: Vec<&Point> = self.points.iter().rev().take(3).collect();
        let circle = match &initial.as_slice() {
            [a, b, c] => Circle::enclose3(a, b, c),
            [a, b] => Circle::enclose3(a, b, b),
            [a] => Circle::enclose3(a, a, a),
            _ => Circle {
                center: Point::origin(),
                radius: 0.0,
            },
        };
        let mut list = PathList::new();
        for point in initial {
            EnclosingCircle::render_ngon(4, 2.0, point, 1.0, &mut list);
        }
        EnclosingCircle::render_circle(&circle, &mut list);
        list
    }

    fn render_ngon(n: u32, radius: f64, position: &Point, alpha: f64, list: &mut PathList) -> () {
        let mut shape = geometry::ngon(n, radius)
            .iter()
            .map(|point| point.add(position))
            .collect();
        list.push(&mut shape, alpha, PathEnd::Closed);
    }

    fn render_circle(circle: &Circle, list: &mut PathList) -> () {
        let n = ((circle.radius * PI / 5.0).floor() as u32).max(24);
        EnclosingCircle::render_ngon(n, circle.radius, &circle.center, 0.5, list);
    }
}
