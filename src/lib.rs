use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct App {
    rng: Pcg32,
    asteroids: Vec<Asteroid>,
}

pub struct Asteroid {
    placement: Placement,
    movement: Movement,
    polygon: Vec<Point>,
}

pub struct Placement {
    position: Point,
    rotation: Radians,
}
pub struct Movement {
    velocity: Vector,
    angular_velocity: Radians,
}

#[repr(C)]
pub struct Point {
    x: f64,
    y: f64,
}

type Vector = Point;

type Radians = f64;

//

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    pub fn from_polar(radius: f64, angle: Radians) -> Self {
        Point {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
        }
    }

    pub fn scale(&mut self, factor: f64) -> &mut Self {
        self.x *= factor;
        self.y *= factor;
        self
    }

    pub fn transformed(&self, matrix: &Matrix) -> Self {
        let x = self.x;
        let y = self.y;
        Point {
            x: x * matrix.a + y * matrix.c + matrix.tx,
            y: x * matrix.b + y * matrix.d + matrix.ty,
        }
    }
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

const WIDTH: f64 = 1200.0;
const HEIGHT: f64 = 900.0;

#[wasm_bindgen]
impl App {
    pub fn new() -> Self {
        let mut rng = Pcg32::seed_from_u64(1979);
        App {
            asteroids: Asteroid::field(&mut rng, WIDTH, HEIGHT, 24),
            rng,
        }
    }

    pub fn step(&mut self, dt: f64) -> () {
        if 0.0 < dt {
            for asteroid in self.asteroids.iter_mut() {
                asteroid.step(WIDTH, HEIGHT, dt);
            }
        }
    }

    pub fn view(&self) -> PathList {
        let mut list = PathList::new();
        for asteroid in self.asteroids.iter() {
            list.push(&mut asteroid.to_path(), 1.0, End::Closed);
        }
        list
    }
}

// matrix

pub struct Matrix {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}

impl Matrix {
    pub fn new(position: &Point, rotation: Radians, scale: f64) -> Self {
        let sine = rotation.sin();
        let cosine = rotation.cos();
        Matrix {
            a: scale * cosine,
            b: scale * sine,
            c: scale * -sine,
            d: scale * cosine,
            tx: position.x,
            ty: position.y,
        }
    }
}

//

fn ngon(n: u32) -> Vec<Point> {
    let n = n.max(3);
    let angle = (2.0 * PI) / (n as f64);

    (0..n)
        .map(|i| Point::from_polar(1.0, angle * (i as f64)))
        .collect()
}

// view

#[repr(C)]
pub struct Path {
    offset: usize,
    length: usize,
}

#[repr(u8)]
pub enum End {
    Open = 0,
    Closed = 1,
}

#[wasm_bindgen]
pub struct PathList {
    paths: Vec<Path>,
    alphas: Vec<f64>,
    ends: Vec<End>,
    points: Vec<Point>,
}

impl PathList {
    pub fn new() -> Self {
        PathList {
            paths: Vec::new(),
            points: Vec::new(),
            alphas: Vec::new(),
            ends: Vec::new(),
        }
    }

    pub fn push(&mut self, points: &mut Vec<Point>, alpha: f64, end: End) -> &mut Self {
        self.paths.push(Path {
            offset: self.points.len(),
            length: points.len(),
        });
        self.alphas.push(alpha);
        self.ends.push(end);
        self.points.append(points);
        self
    }
}

#[wasm_bindgen]
impl PathList {
    pub fn length(&self) -> usize {
        self.paths.len()
    }

    pub fn paths(&self) -> *const Path {
        self.paths.as_ptr()
    }

    pub fn alphas(&self) -> *const f64 {
        self.alphas.as_ptr()
    }

    pub fn ends(&self) -> *const End {
        self.ends.as_ptr()
    }

    pub fn points_length(&self) -> usize {
        self.points.len()
    }

    pub fn points(&self) -> *const Point {
        self.points.as_ptr()
    }
}
