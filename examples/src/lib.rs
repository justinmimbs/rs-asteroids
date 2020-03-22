use core::f64::consts::PI;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::wasm_bindgen;

use app::{
    render,
    render::{PathEnd, PathList},
};
use asteroids::{
    geometry,
    geometry::{Circle, Point, Polygon, Radians, Size, Vector},
    iter::EdgesCycleIterator,
    motion,
    motion::{Collide, Movement, Placement},
    Asteroid, Dispersion, Particle,
};

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

fn render_ngon(n: u32, radius: f64, position: &Point, alpha: f64, list: &mut PathList) -> () {
    let mut shape = geometry::ngon(n, radius)
        .iter()
        .map(|point| point.add(position))
        .collect();
    list.push(&mut shape, alpha, PathEnd::Closed);
}

fn render_circle(circle: &Circle, list: &mut PathList) -> () {
    let n = ((circle.radius * PI / 5.0).floor() as u32).max(24);
    render_ngon(n, circle.radius, &circle.center, 0.5, list);
}

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
        AsteroidField(Asteroid::field(&mut rng, &BOUNDS, count, 0.0))
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
        if let Some((_, velocity)) = &mut self.target {
            *velocity = Vector::new(velocity_x, velocity_y);
        }
    }

    pub fn fire(&mut self) -> () {
        if let Some((position, velocity)) = &self.target {
            let mut particles = Dispersion::new(position.clone(), velocity.clone(), 150.0, 100.0)
                .burst(&mut self.rng, 24);
            self.particles.append(&mut particles);

            let mut pieces = Dispersion::new(position.clone(), velocity.clone(), 100.0, 150.0)
                .explode(
                    &mut self.rng,
                    (Particles::target_shape().iter())
                        .map(|point| point.add(position))
                        .edges_cycle(),
                );
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
            let mut shape = (Particles::target_shape().iter())
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
        let mut list = PathList::new();
        for point in &self.points {
            render_ngon(4, 2.0, point, 1.0, &mut list);
        }
        render_circle(&Circle::enclose(&self.points), &mut list);
        list
    }
}

// 05

#[wasm_bindgen]
pub struct SplitPolygon {
    polygon: Vec<Point>,
    line: (Point, Point),
}

#[wasm_bindgen]
impl SplitPolygon {
    pub fn new() -> Self {
        let center = Point::new(BOUNDS.width, BOUNDS.height).scale(0.5);
        let mut polygon = geometry::ngon(7, 100.0);
        polygon[0] = Point::origin();
        SplitPolygon {
            polygon: polygon.iter().map(|p| p.add(&center)).collect(),
            line: (Point::origin(), Point::origin()),
        }
    }

    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> () {
        self.line = (Point::new(x1, y1), Point::new(x2, y2));
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        let (a, b) = &self.line;
        list.push(&mut vec![a.clone(), b.clone()], 0.5, PathEnd::Open);
        for mut polygon in Polygon(&self.polygon).split(a, b) {
            render_circle(&Circle::enclose(&polygon), &mut list);
            list.push(&mut polygon, 1.0, PathEnd::Closed);
        }
        list
    }
}

// 06

#[wasm_bindgen]
pub struct Impulse {
    pending: Option<(Point, Vector)>,
    polygon: Vec<Point>,
    placement: Placement,
    movement: Movement,
}

#[wasm_bindgen]
impl Impulse {
    pub fn new() -> Self {
        Impulse {
            pending: None,
            polygon: geometry::ngon(7, 60.0),
            placement: Placement {
                position: Point::new(BOUNDS.width, BOUNDS.height).scale(0.5),
                rotation: 0.0,
            },
            movement: Movement {
                velocity: Vector::zero(),
                angular_velocity: 0.0,
            },
        }
    }

    pub fn ready(&mut self, contact_x: f64, contact_y: f64) -> () {
        self.pending = Some((Point::new(contact_x, contact_y), Vector::zero()));
    }

    pub fn aim(&mut self, velocity_x: f64, velocity_y: f64) -> () {
        if let Some((_, velocity)) = &mut self.pending {
            *velocity = Vector::new(velocity_x, velocity_y);
        }
    }

    pub fn fire(&mut self) -> () {
        if let Some((contact, velocity)) = &self.pending {
            let impulse = Movement::from_impulse(&self.placement.position, contact, velocity);
            self.movement = self.movement.add(&impulse);
            self.pending = None;
        }
    }

    pub fn step(&mut self, dt: f64) -> () {
        if dt <= 0.0 {
            return ();
        }
        self.placement
            .apply_movement(&self.movement, dt)
            .wrap_position(&BOUNDS);
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        let mut shape = self.placement.transform_points(&self.polygon);
        list.push(&mut shape, 1.0, PathEnd::Closed);
        if let Some((contact, velocity)) = &self.pending {
            let mut line = vec![contact.clone(), contact.sub(velocity)];
            list.push(&mut line, 0.2, PathEnd::Open);
        }
        list
    }
}

// 07

#[wasm_bindgen]
pub struct Colliding {
    n: usize,
    pair: (Disk, Disk),
}

#[wasm_bindgen]
impl Colliding {
    pub fn new() -> Self {
        Colliding {
            n: 0,
            pair: get_pair(0),
        }
    }

    pub fn next(&mut self) -> () {
        self.n = (self.n + 1) % (SPEC_PAIRS.len() * RADIUS_PAIRS.len());
        self.pair = get_pair(self.n);
    }

    pub fn step(&mut self, dt: f64) -> () {
        if dt <= 0.0 {
            return ();
        }
        self.pair.0.step(dt);
        self.pair.1.step(dt);
        if let Some((_, a, b)) = motion::collide(&self.pair.0, &self.pair.1, 0.9) {
            (self.pair.0).movement = a;
            (self.pair.1).movement = b;
        }
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        list.push(&mut (self.pair.0).boundary(), 1.0, PathEnd::Closed);
        list.push(&mut (self.pair.1).boundary(), 1.0, PathEnd::Closed);
        list
    }
}

fn get_pair(n: usize) -> (Disk, Disk) {
    let i = n % SPEC_PAIRS.len();
    let j = (n / SPEC_PAIRS.len()) % RADIUS_PAIRS.len();
    let (spec1, spec2) = SPEC_PAIRS[i];
    let (radius1, radius2) = RADIUS_PAIRS[j];

    (to_disk(spec1, radius1), to_disk(spec2, radius2))
}

type DiskSpec = ((f64, f64), (f64, f64), f64);

fn to_disk(spec: DiskSpec, radius: f64) -> Disk {
    let (x, y) = spec.0;
    let (vx, vy) = spec.1;
    let av = spec.2;
    Disk::new(Point::new(x, y), Vector::new(vx, vy), av, radius)
}

const SPEC_PAIRS: [(DiskSpec, DiskSpec); 8] = [
    (
        ((300.0, 100.0), (300.0, 300.0), PI / 3.0),
        ((600.0, 400.0), (0.0, 0.0), 0.0),
    ),
    (
        ((100.0, 100.0), (300.0, 300.0), (PI / 3.0)),
        ((300.0, 300.0), (200.0, 200.0), 0.0),
    ),
    (
        ((300.0, 100.0), (300.0, 330.0), 0.0),
        ((600.0, 400.0), (0.0, 0.0), 0.0),
    ),
    (
        ((300.0, 100.0), (300.0, 430.0), (PI / 3.0)),
        ((600.0, 400.0), (0.0, 0.0), 0.0),
    ),
    (
        ((300.0, 100.0), (300.0, 300.0), (PI / 3.0)),
        ((400.0, 600.0), (200.0, -200.0), 0.0),
    ),
    (
        ((300.0, 100.0), (350.0, 50.0), (PI / 8.0)),
        ((900.0, 100.0), (-150.0, 70.0), (PI / -2.0)),
    ),
    (
        ((300.0, 100.0), (250.0, 100.0), (PI / 3.0)),
        ((900.0, 100.0), (-250.0, 100.0), (PI / -3.0)),
    ),
    (
        ((300.0, 100.0), (350.0, 270.0), (PI / -5.0)),
        ((900.0, 700.0), (-150.0, -150.0), (PI / 2.0)),
    ),
];

const RADIUS_PAIRS: [(f64, f64); 3] = [(50.0, 50.0), (50.0, 30.0), (30.0, 50.0)];

struct Disk {
    placement: Placement,
    movement: Movement,
    radius: f64,
    polygon: Vec<Point>,
}

impl Disk {
    fn new(position: Point, velocity: Vector, angular_velocity: Radians, radius: f64) -> Self {
        Disk {
            placement: Placement {
                position,
                rotation: 0.0,
            },
            movement: Movement {
                velocity,
                angular_velocity,
            },
            polygon: geometry::ngon(12, radius),
            radius,
        }
    }
    fn step(&mut self, dt: f64) -> () {
        self.placement
            .apply_movement(&self.movement, dt)
            .wrap_position(&BOUNDS);
    }
}

impl Collide for Disk {
    fn center(&self) -> &Point {
        &self.placement.position
    }
    fn radius(&self) -> f64 {
        self.radius
    }
    fn boundary(&self) -> Vec<Point> {
        self.placement.transform_points(&self.polygon)
    }
    fn movement(&self) -> &Movement {
        &self.movement
    }
    fn mass(&self) -> f64 {
        self.radius * self.radius
    }
}
