use std::array;

use crate::transform::position::{MouseTransform, Position, RayIterator};

#[derive(Clone, Debug)]
pub struct ValueMap<const N: usize, T: Sized> {
    values: [[T; N]; N],
}

impl<const N: usize, T: Sized> ValueMap<N, T> {
    pub fn new(fill_with: T) -> Self
    where
        T: Clone,
    {
        Self {
            values: array::from_fn(|_col_num| array::from_fn(|_row_num| fill_with.clone())),
        }
    }

    pub fn value(&self, position: Position) -> Option<&T> {
        let x = position.x as usize;
        let y = position.y as usize;
        if x >= N || y >= N {
            return None;
        }
        Some(&self.values[x][y])
    }
    pub fn value_mut(&mut self, position: Position) -> Option<&mut T> {
        let x = position.x as usize;
        let y = position.y as usize;
        if x >= N || y >= N {
            return None;
        }
        Some(&mut self.values[x][y])
    }

    pub fn depth_while<F: Fn(&T) -> bool>(&self, f: F, from: MouseTransform) -> usize {
        let ray_iter = RayIterator::<N>::new(from.pos, from.dir).enumerate().skip(1);

        let mut check_next_i = 0;

        for (depth_if_continuing_here, pos) in ray_iter {
            let val = self.value(pos.pos).expect("Ray iter should be in bounds");

            let continuing = f(val);

            if !continuing {
                return check_next_i - 1;
            }
            check_next_i = depth_if_continuing_here;

        }

        check_next_i
    }
}
