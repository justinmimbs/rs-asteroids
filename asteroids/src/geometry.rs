use std::f64;
use std::f64::consts::PI;

pub type Radians = f64;

pub struct Size {
    pub width: f64,
    pub height: f64,
}

// Point/Vector

#[repr(C)]
#[derive(Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub type Vector = Point;

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    pub fn origin() -> Self {
        Point { x: 0.0, y: 0.0 }
    }

    pub fn zero() -> Self {
        Point { x: 0.0, y: 0.0 }
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

    pub fn midpoint(&self, other: &Point) -> Self {
        Point {
            x: (self.x + other.x) * 0.5,
            y: (self.y + other.y) * 0.5,
        }
    }

    pub fn distance_squared(&self, other: &Point) -> f64 {
        (other.x - self.x).powi(2) + (other.y - self.y).powi(2)
    }

    pub fn distance(&self, other: &Point) -> f64 {
        self.distance_squared(other).sqrt()
    }

    pub fn translate(&self, distance: f64, angle: Radians) -> Self {
        Point {
            x: self.x + distance * angle.cos(),
            y: self.y + distance * angle.sin(),
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

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn angle(&self) -> Radians {
        self.y.atan2(self.x)
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

pub fn ngon(n: u32, radius: f64) -> Vec<Point> {
    let n = n.max(3);
    let angle = (2.0 * PI) / (n as f64);

    (0..n)
        .map(|i| Point::from_polar(radius, angle * (i as f64)))
        .collect()
}

// lines

fn perpendicular_bisector(a: &Point, b: &Point) -> (Point, Point) {
    let m = a.midpoint(b);
    (m.clone(), Point::new(m.x + (b.y - a.y), m.y - (b.x - a.x)))
}

enum Inter {
    LineLine,
    LineSegment,
    SegmentSegment,
}

fn intersect(inter: Inter, a: &Point, b: &Point, c: &Point, d: &Point) -> Option<(Point)> {
    let rx = b.x - a.x;
    let ry = b.y - a.y;
    let sx = d.x - c.x;
    let sy = d.y - c.y;
    let rs = rx * sy - sx * ry; // cross r s
    let ex = c.x - a.x;
    let ey = c.y - a.y;
    let u = (ex * ry - rx * ey) / rs; // cross e r / rs
    if rs != 0.0 {
        match inter {
            Inter::LineLine => Some(Point::new(c.x + u * sx, c.y + u * sy)),
            Inter::LineSegment => {
                if 0.0 <= u && u <= 1.0 {
                    Some(Point::new(c.x + u * sx, c.y + u * sy))
                } else {
                    None
                }
            }
            Inter::SegmentSegment => {
                if 0.0 <= u && u <= 1.0 {
                    let t = (ex * sy - sx * ey) / rs; // cross e s / rs
                    if 0.0 <= t && t <= 1.0 {
                        Some(Point::new(c.x + u * sx, c.y + u * sy))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    } else {
        None
    }
}

// circles

fn circumcenter3(a: &Point, b: &Point, c: &Point) -> Option<Point> {
    let (p1, p2) = perpendicular_bisector(a, b);
    let (p3, p4) = perpendicular_bisector(a, c);
    intersect(Inter::LineLine, &p1, &p2, &p3, &p4)
}

// Circ

struct Circ {
    center: Point,
    radius_squared: f64,
}

impl Circ {
    fn is_enclosed(&self, point: &Point) -> bool {
        // `self.center.distance_squared(point) <= self.radius_squared` is the ideal comparison
        self.center.distance_squared(point) - self.radius_squared
            <= f64::EPSILON * 10.0 * self.radius_squared
    }

    fn degenerate(center: Point) -> Self {
        Circ {
            radius_squared: 0.0,
            center,
        }
    }

    fn circumcircle2(a: &Point, b: &Point) -> Self {
        let center = a.midpoint(b);
        Circ {
            radius_squared: a.distance_squared(&center),
            center,
        }
    }

    fn circumcircle3(a: &Point, b: &Point, c: &Point) -> Option<Self> {
        circumcenter3(a, b, c).map(|center| Circ {
            radius_squared: a.distance_squared(&center),
            center,
        })
    }

    fn enclose3(a: &Point, b: &Point, c: &Point) -> Self {
        let ab = Circ::circumcircle2(a, b);
        if ab.is_enclosed(c) {
            return ab;
        }
        let ac = Circ::circumcircle2(a, c);
        if ac.is_enclosed(b) {
            return ac;
        }
        let bc = Circ::circumcircle2(b, c);
        if bc.is_enclosed(a) {
            return bc;
        }
        Circ::circumcircle3(a, b, c).unwrap_or_else(|| Circ::degenerate(a.clone()))
    }
}

// Circle

pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    pub fn enclose3(a: &Point, b: &Point, c: &Point) -> Self {
        let circ = Circ::enclose3(a, b, c);
        Circle {
            center: circ.center,
            radius: circ.radius_squared.sqrt(),
        }
    }
}
