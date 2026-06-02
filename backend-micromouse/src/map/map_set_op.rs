use crate::{
    map::map::{CellDiscoveryStatus, Map, WallDiscoveryStatus},
    transform::{direction::Direction, position::Position},
};

// The logical conclusion of self and other (if they aren't contradictory)
pub trait Union
where
    Self: Sized,
{
    fn union(&self, other: &Self) -> Option<Self>;
}

// The largest possible thing self and other have in common
pub trait Intersect
where
    Self: Sized,
{
    fn intersect(&self, other: &Self) -> Self;
}

impl Union for CellDiscoveryStatus {
    fn union(&self, other: &Self) -> Option<Self> {
        Some(match (self, other) {
            (CellDiscoveryStatus::Undiscovered, o) => *o,
            (o, CellDiscoveryStatus::Undiscovered) => *o,
            (CellDiscoveryStatus::Discovered, CellDiscoveryStatus::Visited) => {
                CellDiscoveryStatus::Visited
            }
            (CellDiscoveryStatus::Visited, CellDiscoveryStatus::Discovered) => {
                CellDiscoveryStatus::Visited
            }
            (CellDiscoveryStatus::Discovered, CellDiscoveryStatus::Discovered) => {
                CellDiscoveryStatus::Discovered
            }
            (CellDiscoveryStatus::Visited, CellDiscoveryStatus::Visited) => {
                CellDiscoveryStatus::Visited
            }
        })
    }
}

impl Intersect for CellDiscoveryStatus {
    fn intersect(&self, other: &Self) -> Self {
        match (self, other) {
            (CellDiscoveryStatus::Undiscovered, _) => CellDiscoveryStatus::Undiscovered,
            (_, CellDiscoveryStatus::Undiscovered) => CellDiscoveryStatus::Undiscovered,
            (CellDiscoveryStatus::Discovered, _) => CellDiscoveryStatus::Discovered,
            (_, CellDiscoveryStatus::Discovered) => CellDiscoveryStatus::Discovered,
            (CellDiscoveryStatus::Visited, CellDiscoveryStatus::Visited) => {
                CellDiscoveryStatus::Visited
            }
        }
    }
}

impl Union for WallDiscoveryStatus {
    fn union(&self, other: &Self) -> Option<Self> {
        match (self, other) {
            // Anything can be upgraded out of Undiscovered
            (WallDiscoveryStatus::Undiscovered, o) => Some(*o),
            (o, WallDiscoveryStatus::Undiscovered) => Some(*o),

            // Visited includes being walkthrough
            (WallDiscoveryStatus::Exists(false), WallDiscoveryStatus::Visited) => {
                Some(WallDiscoveryStatus::Visited)
            }
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Exists(false)) => {
                Some(WallDiscoveryStatus::Visited)
            }

            // Both sides equal
            (WallDiscoveryStatus::Exists(false), WallDiscoveryStatus::Exists(false)) => {
                Some(WallDiscoveryStatus::Exists(false))
            }
            (WallDiscoveryStatus::Exists(true), WallDiscoveryStatus::Exists(true)) => {
                Some(WallDiscoveryStatus::Exists(true))
            }
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Visited) => {
                Some(WallDiscoveryStatus::Visited)
            }

            (_, _) => None,
        }
    }
}

impl Intersect for WallDiscoveryStatus {
    fn intersect(&self, other: &Self) -> Self {
        match (self, other) {
            (WallDiscoveryStatus::Undiscovered, _) => WallDiscoveryStatus::Undiscovered,
            (_, WallDiscoveryStatus::Undiscovered) => WallDiscoveryStatus::Undiscovered,
            (WallDiscoveryStatus::Exists(true), WallDiscoveryStatus::Visited) => {
                WallDiscoveryStatus::Undiscovered
            }
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Exists(true)) => {
                WallDiscoveryStatus::Undiscovered
            }
            (WallDiscoveryStatus::Exists(false), WallDiscoveryStatus::Visited) => {
                WallDiscoveryStatus::Exists(false)
            }
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Exists(false)) => {
                WallDiscoveryStatus::Exists(false)
            }
            (WallDiscoveryStatus::Exists(true), WallDiscoveryStatus::Exists(false)) => {
                WallDiscoveryStatus::Undiscovered
            }
            (WallDiscoveryStatus::Exists(false), WallDiscoveryStatus::Exists(true)) => {
                WallDiscoveryStatus::Undiscovered
            }
            (WallDiscoveryStatus::Exists(true), WallDiscoveryStatus::Exists(true)) => {
                WallDiscoveryStatus::Exists(true)
            }
            (WallDiscoveryStatus::Exists(false), WallDiscoveryStatus::Exists(false)) => {
                WallDiscoveryStatus::Exists(false)
            }
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Visited) => {
                WallDiscoveryStatus::Visited
            }
        }
    }
}

impl<const N: usize> Union for Map<N> {
    fn union(&self, other: &Self) -> Option<Self> {
        let mut union_map = Map::new();
        for x in 0..N {
            for y in 0..N {
                let pos = Position {
                    x: x as u32,
                    y: y as u32,
                };
                let wall_self_r = self
                    .wall(&pos, &Direction::PosX)
                    .expect("Impossible, bound by iterator");
                let wall_other_r = other
                    .wall(&pos, &Direction::PosX)
                    .expect("Impossible, bound by iterator");
                *(union_map
                    .wall_mut(&pos, &Direction::PosX)
                    .expect("Impossible, bound by iterator")) = wall_self_r.union(wall_other_r)?;

                let wall_self_d = self
                    .wall(&pos, &Direction::PosY)
                    .expect("Impossible, bound by iterator");
                let wall_other_d = other
                    .wall(&pos, &Direction::PosY)
                    .expect("Impossible, bound by iterator");
                *(union_map
                    .wall_mut(&pos, &Direction::PosY)
                    .expect("Impossible, bound by iterator")) = wall_self_d.union(wall_other_d)?;

                let cell_self = self.cell(&pos).expect("Impossible, bound by iterator");
                let cell_other = other.cell(&pos).expect("Impossible, bound by iterator");
                *(union_map
                    .cell_mut(&pos)
                    .expect("Impossible, bound by iterator")) = cell_self.union(cell_other)?;
            }
        }
        Some(union_map)
    }
}

impl<const N: usize> Intersect for Map<N> {
    fn intersect(&self, other: &Self) -> Self {
        let mut intersect_map = Map::new();
        for x in 0..N {
            for y in 0..N {
                let pos = Position {
                    x: x as u32,
                    y: y as u32,
                };
                let wall_self_r = self
                    .wall(&pos, &Direction::PosX)
                    .expect("Impossible, bound by iterator");
                let wall_other_r = other
                    .wall(&pos, &Direction::PosX)
                    .expect("Impossible, bound by iterator");
                *(intersect_map
                    .wall_mut(&pos, &Direction::PosX)
                    .expect("Impossible, bound by iterator")) = wall_self_r.intersect(wall_other_r);

                let wall_self_d = self
                    .wall(&pos, &Direction::PosY)
                    .expect("Impossible, bound by iterator");
                let wall_other_d = other
                    .wall(&pos, &Direction::PosY)
                    .expect("Impossible, bound by iterator");
                *(intersect_map
                    .wall_mut(&pos, &Direction::PosY)
                    .expect("Impossible, bound by iterator")) = wall_self_d.intersect(wall_other_d);

                let cell_self = self.cell(&pos).expect("Impossible, bound by iterator");
                let cell_other = other.cell(&pos).expect("Impossible, bound by iterator");
                *(intersect_map
                    .cell_mut(&pos)
                    .expect("Impossible, bound by iterator")) = cell_self.intersect(cell_other);
            }
        }
        intersect_map
    }
}
