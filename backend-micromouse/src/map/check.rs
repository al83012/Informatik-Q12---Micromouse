use crate::{
    comm::micromouse_message::TransformedCommandResult,
    map::map::{CellDiscoveryStatus, Map, PartialMap, WallDiscoveryStatus},
    transform::position::Position,
    map::world_data::{PartialWorldData, WorldData},
};

pub trait PotentiallyEq {
    type Other;
    fn potentially_eq(&self, other: &Self::Other) -> bool;
}

impl PotentiallyEq for WallDiscoveryStatus {
    type Other = Self;
    fn potentially_eq(&self, other: &Self::Other) -> bool {
        match (self, other) {
            (WallDiscoveryStatus::Undiscovered, _) | (_, WallDiscoveryStatus::Undiscovered) => true,
            (WallDiscoveryStatus::Exists(a), WallDiscoveryStatus::Exists(b)) => a == b,
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Exists(false)) => true,
            (WallDiscoveryStatus::Exists(false), WallDiscoveryStatus::Visited) => true,
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Visited) => true,
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Exists(true)) => false,
            (WallDiscoveryStatus::Exists(true), WallDiscoveryStatus::Visited) => false,
        }
    }
}

impl<const N: usize> PotentiallyEq for Map<N> {
    type Other = PartialMap<N>;

    fn potentially_eq(&self, other: &Self::Other) -> bool {
        let inner_map = other.0;
        for x in 0..N {
            for y in 0..N {
                let pos = Position {
                    x: x as u32,
                    y: y as u32,
                };
                let a1 = self.wall(&pos, &crate::transform::direction::Direction::PosX);
                if a1.is_none() {
                    return false;
                }
                let a1 = a1.unwrap();

                let a2 = self.wall(&pos, &crate::transform::direction::Direction::PosY);
                if a2.is_none() {
                    return false;
                }
                let a2 = a2.unwrap();

                let b1 = inner_map.wall(&pos, &crate::transform::direction::Direction::PosX);
                if b1.is_none() {
                    return false;
                }
                let b1 = b1.unwrap();

                let b2 = inner_map.wall(&pos, &crate::transform::direction::Direction::PosY);
                if b2.is_none() {
                    return false;
                }
                let b2 = b2.unwrap();

                if !(a1.potentially_eq(b1) && a2.potentially_eq(b2)) {
                    return false;
                }
            }
        }
        true
    }
}

impl<const N: usize> PotentiallyEq for WorldData<N> {
    type Other = PartialWorldData<N>;

    fn potentially_eq(&self, other: &Self::Other) -> bool {
        if !self.map.potentially_eq(&other.map()) {
            return false;
        }
        if self.mouse != other.mouse {
            return false;
        }

        true
    }
}
