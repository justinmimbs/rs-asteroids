use std::f64::consts::PI;

pub type Radians = f64;

pub struct Size {
    pub width: f64,
    pub height: f64,
}

// Point/Vector

#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub type Vector = Point;

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

    pub fn scale(&self, factor: f64) -> Self {
        Point {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    pub fn add(&self, other: &Point) -> Self {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn sub(&self, other: &Point) -> Self {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn transform(&self, matrix: &Matrix) -> Self {
        let x = self.x;
        let y = self.y;
        Point {
            x: x * matrix.a + y * matrix.c + matrix.tx,
            y: x * matrix.b + y * matrix.d + matrix.ty,
        }
    }
}

// Matrix

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

// polygons

pub fn ngon(n: u8, radius: f64) -> Vec<Point> {
    let n = n.max(3);
    let angle = (2.0 * PI) / (n as f64);

    (0..n)
        .map(|i| Point::from_polar(radius, angle * (i as f64)))
        .collect()
}
