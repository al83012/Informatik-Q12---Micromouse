use crate::{map::map::{PartialMap, WallDiscoveryStatus}, transform::{direction::Direction, position::Position}};





/// This turns a partial map into one, where all the boundaries between the discovered and
/// undiscovered areas are blocked off with walls
pub fn as_capsule_map<const N: usize>(mut map: PartialMap<N>) -> PartialMap<N> {

    for x in 0..N {
        for y in 0..N {
            let pos = Position {
                x: x as u32,
                y: y as u32
            };

            let cell = *map.cell(&pos).expect("In bounds");
            if x != N - 1 {
                let pos_right = Position {
                    x: x as u32 + 1,
                    y: y as u32
                };

                let cell_right = *map.cell(&pos_right).expect("In bounds");

                if cell.is_discovered() ^ cell_right.is_discovered() {
                    *(map.wall_mut(&pos, &Direction::PosX).expect("In bounds")) = WallDiscoveryStatus::Exists(true);
                }
            }

            if y != N-1 {
                let pos_down = Position {
                    x: x as u32,
                    y: y as u32 + 1
                };
                let cell_down = *map.cell(&pos_down).expect("In bounds");

                if cell.is_discovered() ^ cell_down.is_discovered() {
                    *(map.wall_mut(&pos, &Direction::PosY).expect("In bounds")) = WallDiscoveryStatus::Exists(true);
                }
            }
        }
    }

    map
}
