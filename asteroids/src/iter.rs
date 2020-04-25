// iterator adaptor: edges_cycle

pub struct EdgesCycle<I, T> {
    iter: I,
    state: Option<(T, T)>,
}

impl<I> Iterator for EdgesCycle<I, I::Item>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.iter.next() {
            match &mut self.state {
                None => {
                    // initialize
                    self.state = Some((next.clone(), next));
                    self.next()
                }
                Some((_first, previous)) => {
                    // continue
                    let edge = (previous.clone(), next.clone());
                    *previous = next;
                    Some(edge)
                }
            }
        } else {
            match &mut self.state {
                Some((first, last)) => {
                    // finalize
                    let edge = (last.clone(), first.clone());
                    self.state = None;
                    Some(edge)
                }
                None => {
                    // empty
                    None
                }
            }
        }
    }
}

// trait with edges_cycle method

pub trait EdgesCycleIterator: Sized + Iterator {
    fn edges_cycle(self) -> EdgesCycle<Self, Self::Item>;
}

// _blanket implementation_ of EdgesCycleIterator for all types implementing Iterator

impl<I: Iterator> EdgesCycleIterator for I {
    fn edges_cycle(self) -> EdgesCycle<Self, Self::Item> {
        EdgesCycle {
            iter: self,
            state: None,
        }
    }
}

#[cfg(test)]
mod test_edges_cycle {
    use super::*;

    #[test]
    fn test_empty() {
        let mut iter = (0..0).edges_cycle();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_length_1() {
        let mut iter = (0..1).edges_cycle();
        assert_eq!(iter.next(), Some((0, 0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_length_2() {
        let mut iter = (0..2).edges_cycle();
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), Some((1, 0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_length_3() {
        let mut iter = (0..3).edges_cycle();
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), Some((1, 2)));
        assert_eq!(iter.next(), Some((2, 0)));
        assert_eq!(iter.next(), None);
    }
}

// iterator adaptor: edges

pub struct Edges<I, T> {
    iter: I,
    previous: Option<T>,
}

impl<I> Iterator for Edges<I, I::Item>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.iter.next() {
            match &mut self.previous {
                None => {
                    // initialize
                    self.previous = Some(next);
                    self.next()
                }
                Some(previous) => {
                    // continue
                    let edge = (previous.clone(), next.clone());
                    *previous = next;
                    Some(edge)
                }
            }
        } else {
            None
        }
    }
}

// trait with edges method

pub trait EdgesIterator: Sized + Iterator {
    fn edges(self) -> Edges<Self, Self::Item>;
}

// _blanket implementation_ of EdgesIterator for all types implementing Iterator

impl<I: Iterator> EdgesIterator for I {
    fn edges(self) -> Edges<Self, Self::Item> {
        Edges {
            iter: self,
            previous: None,
        }
    }
}

#[cfg(test)]
mod test_edges {
    use super::*;

    #[test]
    fn test_empty() {
        let mut iter = (0..0).edges();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_length_1() {
        let mut iter = (0..1).edges();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_length_2() {
        let mut iter = (0..2).edges();
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_length_3() {
        let mut iter = (0..3).edges();
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), Some((1, 2)));
        assert_eq!(iter.next(), None);
    }
}

// iterator adaptor: max_length

use crate::geometry::Point;
use core::borrow::Borrow;

pub struct MaxLength<I> {
    iter: I,
    length: f64,
    buffer: Vec<(Point, Point)>,
}

impl<I, P> Iterator for MaxLength<I>
where
    I: Iterator<Item = (P, P)>,
    P: Borrow<Point>,
{
    type Item = (Point, Point);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(segment) = self.buffer.pop() {
            Some(segment)
        } else if let Some((a, b)) = self.iter.next() {
            let a = a.borrow().clone();
            let b = b.borrow().clone();
            let length = a.distance(&b);
            if length <= self.length {
                Some((a, b))
            } else {
                let n = (length / self.length).ceil();
                // build buffer in reverse order
                self.buffer = (0..n as u32)
                    .map(|i| {
                        let x = a.interpolate(&b, (n - (i + 1) as f64) / n);
                        let y = a.interpolate(&b, (n - i as f64) / n);
                        (x, y)
                    })
                    .collect::<Vec<_>>();
                self.buffer.pop()
            }
        } else {
            None
        }
    }
}

// trait with max_length method

pub trait MaxLengthIterator: Sized + Iterator {
    fn max_length(self, length: f64) -> MaxLength<Self>;
}

// _blanket implementation_ of MaxLengthIterator for all types implementing Iterator<Item=(Point, Point)>

impl<I, P> MaxLengthIterator for I
where
    I: Iterator<Item = (P, P)>,
    P: Borrow<Point>,
{
    fn max_length(self, length: f64) -> MaxLength<Self> {
        MaxLength {
            iter: self,
            length,
            buffer: Vec::new(),
        }
    }
}

#[cfg(test)]
mod test_max_length {
    use super::*;

    fn point(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn test_owned_points() {
        let segments: Vec<(Point, Point)> = vec![];
        let mut iter = segments.into_iter().max_length(10.0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_borrowed_points() {
        let segments: Vec<(&Point, &Point)> = vec![];
        let mut iter = segments.into_iter().max_length(10.0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_break_0() {
        let points = vec![point(0.0, 0.0), point(10.0, 0.0)];
        let mut iter = points.iter().edges().max_length(10.0);
        assert_eq!(iter.next(), Some((point(0.0, 0.0), point(10.0, 0.0))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_break_1() {
        let points = vec![point(0.0, 0.0), point(12.0, 0.0)];
        let mut iter = points.iter().edges().max_length(10.0);
        assert_eq!(iter.next(), Some((point(0.0, 0.0), point(6.0, 0.0))));
        assert_eq!(iter.next(), Some((point(6.0, 0.0), point(12.0, 0.0))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_break_2() {
        let points = vec![point(0.0, 0.0), point(24.0, 0.0)];
        let mut iter = points.iter().edges().max_length(10.0);
        assert_eq!(iter.next(), Some((point(0.0, 0.0), point(8.0, 0.0))));
        assert_eq!(iter.next(), Some((point(8.0, 0.0), point(16.0, 0.0))));
        assert_eq!(iter.next(), Some((point(16.0, 0.0), point(24.0, 0.0))));
        assert_eq!(iter.next(), None);
    }
}
