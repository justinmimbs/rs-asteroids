#[derive(Clone)]
pub struct Timer(f64);

impl Timer {
    pub fn new(interval: f64) -> Self {
        Timer(interval.max(0.0))
    }

    pub fn step(&mut self, dt: f64) -> &mut Self {
        if dt < self.0 {
            self.0 -= dt;
        } else if self.0 < dt {
            self.0 = 0.0;
        }
        self
    }

    pub fn is_elapsed(&self) -> bool {
        self.0 <= 0.0
    }
}
