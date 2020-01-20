use std::cmp::Ordering;
use std::f64;
use std::f64::consts::PI;
use std::mem;

use crate::iter::EdgesCycleIterator;

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

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x.partial_cmp(&other.x) {
            None | Some(Ordering::Equal) => match self.y.partial_cmp(&other.y) {
                None => Ordering::Equal,
                Some(ordering) => ordering,
            },
            Some(ordering) => ordering,
        }
    }
}
impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Point {}

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

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    /// Returns directed angle, within range [-PI, PI].

    pub fn angle(&self) -> Radians {
        self.y.atan2(self.x)
    }

    pub fn normalize(&self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Vector::zero()
        } else {
            Vector {
                x: self.x / length,
                y: self.y / length,
            }
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

    pub fn dot(&self, other: &Point) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(&self, other: &Point) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn interpolate(&self, other: &Point, t: f64) -> Self {
        self.scale(1.0 - t).add(&other.scale(t))
    }

    /// Returns unit vector in the direction from self to other.

    pub fn direction_to(&self, other: &Point) -> Self {
        other.sub(self).normalize()
    }

    /// Returns directed angle from self to other, within range [-PI, PI].

    pub fn angle_to(&self, other: &Vector) -> f64 {
        let a = self.normalize();
        let b = other.normalize();
        a.cross(&b).atan2(a.dot(&b))
    }

    /// Returns undirected angle between self and other, within range [0, PI].
    /// Equivalent to `self.angle_to(other).abs()`.

    pub fn angle_between(&self, other: &Vector) -> f64 {
        self.normalize().dot(&other.normalize()).acos()
    }

    pub fn reflect(&self, normal: &Vector) -> Self {
        self.sub(&normal.scale(2.0 * self.dot(normal)))
    }

    pub fn mean(points: &Vec<Point>) -> Option<Point> {
        match points.len() {
            0 => None,
            n => {
                let factor = 1.0 / n as f64;
                let mut result = Point::zero();
                for point in points {
                    result.x += point.x * factor;
                    result.y += point.y * factor;
                }
                Some(result)
            }
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

pub fn ngon(n: u32, radius: f64) -> Vec<Point> {
    let n = n.max(3);
    let angle = (2.0 * PI) / (n as f64);

    (0..n)
        .map(|i| Point::from_polar(radius, angle * (i as f64)))
        .collect()
}

pub struct Polygon<'a>(pub &'a Vec<Point>);

impl Polygon<'_> {
    /// Split a polygon by a line.
    /// Assumes polygon is neither spiral nor self-intersecting.

    pub fn split(self, a: &Point, b: &Point) -> Vec<Vec<Point>> {
        polygons_from_split_points(rotate_split_points(&mut split_points(self.0, a, b)))
    }

    pub fn intersections<'a, T>(self, segments: T) -> Vec<Point>
    where
        T: IntoIterator<Item = (&'a Point, &'a Point)>,
    {
        segments
            .into_iter()
            .flat_map(|a| {
                (self.0)
                    .iter()
                    .edges_cycle()
                    .filter_map(move |b| intersect(Inter::SegmentSegment, a.0, a.1, b.0, b.1))
            })
            .collect()
    }
}

// 1. Insert intersection points.

enum SplitPoint {
    Point(Point),
    Intersection(Point),
}

fn split_points(polygon: &Vec<Point>, a: &Point, b: &Point) -> Vec<SplitPoint> {
    polygon
        .iter()
        .edges_cycle()
        .fold(Vec::new(), |mut points, edge| {
            points.push(SplitPoint::Point(edge.0.clone()));
            if let Some(intersection) = intersect(Inter::LineSegment, a, b, edge.0, edge.1) {
                points.push(SplitPoint::Intersection(intersection));
            }
            points
        })
}

// 2. Rotate split points so that intersection points are in sorted order.
// (This is only necessary if there are more than two intersection points.)

fn rotate_split_points(points: &mut Vec<SplitPoint>) -> &mut Vec<SplitPoint> {
    let intersections: Vec<(usize, &Point)> = points
        .iter()
        .enumerate()
        .filter_map(|(i, split_point)| {
            if let SplitPoint::Intersection(point) = split_point {
                Some((i, point))
            } else {
                None
            }
        })
        .collect();
    if 2 < intersections.len() {
        let mut pairs = intersections.iter().edges_cycle();
        let ord = pairs.next().map(|(a, b)| a.1.cmp(b.1)).unwrap();
        let first = (pairs.find(|(a, b)| a.1.cmp(b.1) != ord).unwrap().1).0;
        points.rotate_left(first);
    }
    points
}

// 3. Construct polygons.

fn polygons_from_split_points(points: &Vec<SplitPoint>) -> Vec<Vec<Point>> {
    let mut working = Vec::new();
    let mut waiting = Vec::new();
    let mut completed = Vec::new();

    for split_point in points.iter() {
        match split_point {
            SplitPoint::Point(point) => {
                working.push(point.clone());
            }
            SplitPoint::Intersection(point) => {
                working.push(point.clone());
                waiting.push(point.clone());
                if waiting.len() != 1 {
                    // if waiting was not empty, then working is complete
                    completed.push(working);
                    working = Vec::new();
                }
                mem::swap(&mut working, &mut waiting);
            }
        }
    }
    completed.push(working);
    completed
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

fn intersect(inter: Inter, a: &Point, b: &Point, c: &Point, d: &Point) -> Option<Point> {
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
    fn encloses(&self, point: &Point) -> bool {
        // `self.center.distance_squared(point) <= self.radius_squared` is the ideal comparison
        self.center.distance_squared(point) - self.radius_squared
            <= f64::EPSILON * 10.0 * self.radius_squared
    }

    fn degenerate(center: &Point) -> Self {
        Circ {
            radius_squared: 0.0,
            center: center.clone(),
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
        if ab.encloses(c) {
            return ab;
        }
        let ac = Circ::circumcircle2(a, c);
        if ac.encloses(b) {
            return ac;
        }
        let bc = Circ::circumcircle2(b, c);
        if bc.encloses(a) {
            return bc;
        }
        Circ::circumcircle3(a, b, c).unwrap_or_else(|| Circ::degenerate(a))
    }
}

// Circle

pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    /// Compute the smallest enclosing circle from a list of points in linear time.
    /// Based on [Emo Welzl's algorithm](https://www.inf.ethz.ch/personal/emo/PublFiles/SmallEnclDisk_LNCS555_91.pdf).
    /// This version assumes the list is already in a random order.

    pub fn enclose(points: &Vec<Point>) -> Self {
        let mut refs: Vec<&Point> = points.iter().collect();
        let points = refs.as_mut_slice();
        let circ = match points.len() {
            0 => Circ::degenerate(&Point::origin()),
            1 => Circ::degenerate(points[0]),
            2 => Circ::circumcircle2(points[0], points[1]),
            _ => Circle::enclose_help(Circ::enclose3(points[0], points[1], points[2]), points, 3),
        };
        Circle {
            center: circ.center,
            radius: circ.radius_squared.sqrt(),
        }
    }

    fn enclose_help(circ: Circ, points: &mut [&Point], i: usize) -> Circ {
        if points.len() <= i {
            circ
        } else {
            if circ.encloses(points[i]) {
                Circle::enclose_help(circ, points, i + 1)
            } else {
                points.swap(3, i);
                points.swap(2, 3);
                points.swap(1, 2);
                points.swap(0, 1);
                Circle::enclose_help(Circ::enclose3(points[0], points[1], points[2]), points, 3)
            }
        }
    }
}
