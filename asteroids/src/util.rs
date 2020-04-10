pub struct Timer(f64);

impl Timer {
    pub fn new(duration: f64) -> Self {
        Timer(duration.max(0.0))
    }

    pub fn step(&mut self, dt: f64) -> () {
        self.0 -= dt;
    }

    pub fn remaining(&self) -> f64 {
        self.0
    }

    pub fn is_elapsed(&self) -> bool {
        self.0 <= 0.0
    }
}

pub struct Interval {
    period: f64,
    t: f64,
}

impl Interval {
    pub fn new(period: f64, t: f64) -> Self {
        Interval {
            period: period.max(0.0),
            t: t.max(0.0),
        }
    }

    pub fn step(&mut self, dt: f64) -> () {
        self.t += dt;
    }
}

impl Iterator for Interval {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        if self.period <= self.t {
            self.t -= self.period;
            Some(self.t)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test_interval {
    use super::*;

    #[test]
    fn test_init0() {
        let mut iter = Interval::new(2.0, 0.0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_init1() {
        let mut iter = Interval::new(2.0, 2.0);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_init0_step1() {
        let mut iter = Interval::new(2.0, 0.0);
        iter.step(2.0);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_init1_step1() {
        let mut iter = Interval::new(2.0, 2.0);
        iter.step(2.0);
        assert_eq!(iter.next(), Some(2.0));
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_step_accumulate() {
        let mut iter = Interval::new(2.0, 0.0);
        iter.step(1.0);
        assert_eq!(iter.next(), None);
        iter.step(1.0);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_step_many() {
        let mut iter = Interval::new(2.0, 0.0);
        iter.step(8.0);
        assert_eq!(iter.next(), Some(6.0));
        assert_eq!(iter.next(), Some(4.0));
        assert_eq!(iter.next(), Some(2.0));
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_step_many_x2() {
        let mut iter = Interval::new(2.0, 0.0);
        iter.step(5.0);
        assert_eq!(iter.next(), Some(3.0));
        assert_eq!(iter.next(), Some(1.0));
        assert_eq!(iter.next(), None);
        iter.step(3.0);
        assert_eq!(iter.next(), Some(2.0));
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), None);
    }
}
