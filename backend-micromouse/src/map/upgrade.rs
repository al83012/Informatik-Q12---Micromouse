use crate::{map::map::{CellDiscoveryStatus, Map, WallDiscoveryStatus}, transform::{direction::Direction, position::Position}};



pub trait IsUpgradeable {
    fn could_be_upgrade_of(&self, base: &Self) -> bool;
}


impl IsUpgradeable for CellDiscoveryStatus {
    fn could_be_upgrade_of(&self, base: &Self) -> bool {
        match (self, base) {
            // Everything can be upgraded from Undiscovered
            (_, CellDiscoveryStatus::Undiscovered) => true,
            // Visited is the highest upgrade available
            (CellDiscoveryStatus::Visited, _) => true,
            // No upgrade, but not contradictory
            (a, b) if a == b => true,
            _ => false
        }
    }
}

impl IsUpgradeable for WallDiscoveryStatus {
    fn could_be_upgrade_of(&self, base: &Self) -> bool {
        match (self, base) {
            (_, WallDiscoveryStatus::Undiscovered) => true,
            (WallDiscoveryStatus::Visited, WallDiscoveryStatus::Exists(false)) => true,
            (a, b) if a == b => true,
            _ => false
        }
    }
}


impl<const N: usize> IsUpgradeable for Map<N> {
    fn could_be_upgrade_of(&self, base: &Self) -> bool {
        // (do not need to check the NegX and NegY bounds, are always Exists(true))
        for x in 0..N {
            for y in 0..N {
                let pos = Position{x: x as u32, y: y as u32};
                let wall_self_r = self.wall(&pos, &Direction::PosX).expect("Impossible, bound by iterator");
                let wall_base_r = base.wall(&pos, &Direction::PosX).expect("Impossible, bound by iterator");
                if !wall_self_r.could_be_upgrade_of(wall_base_r) {
                    return false;
                }

                let wall_self_d = self.wall(&pos, &Direction::PosY).expect("Impossible, bound by iterator");
                let wall_base_d = base.wall(&pos, &Direction::PosY).expect("Impossible, bound by iterator");
                if !wall_self_d.could_be_upgrade_of(wall_base_d) {
                    return false;
                }

                let cell_self = self.cell(&pos).expect("Impossible, bound by iterator");
                let cell_base = base.cell(&pos).expect("Impossible, bound by iterator");
                if !cell_self.could_be_upgrade_of(cell_base) {
                    return false;
                }
            }
        }
        true
    }
}
