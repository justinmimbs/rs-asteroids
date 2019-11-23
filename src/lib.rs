use std::f64::consts::PI;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct App {
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
}

impl Asteroid {
    pub fn shape() -> Vec<Point> {
        let mut polygon = ngon(5);
        for point in polygon.iter_mut() {
            point.scale(50.0);
        }
        polygon
    }

    pub fn step(&mut self, dt: f64) -> &mut Self {
        self.placement.apply_movement(&self.movement, dt);
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

#[wasm_bindgen]
impl App {
    pub fn new() -> Self {
        App {
            asteroids: vec![Asteroid {
                placement: Placement {
                    position: Point::new(90.0, 90.0),
                    rotation: 0.0,
                },
                movement: Movement {
                    velocity: Point::new(50.0, 50.0),
                    angular_velocity: PI / 2.0,
                },
                polygon: Asteroid::shape(),
            }],
        }
    }

    pub fn step(&mut self, dt: f64) -> () {
        if 0.0 < dt {
            for asteroid in self.asteroids.iter_mut() {
                asteroid.step(dt);
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
        .map(|i| {
            let theta = angle * (i as f64);
            Point::new(theta.cos(), theta.sin())
        })
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
