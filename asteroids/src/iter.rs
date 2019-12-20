// iterator adaptor: edges_cycle

pub struct EdgesCycle<I, T> {
    iter: I,
    state: Option<(T, T)>,
}

impl<I, T> Iterator for EdgesCycle<I, T>
where
    I: Iterator<Item = T>,
    T: Clone,
{
    type Item = (T, T);

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