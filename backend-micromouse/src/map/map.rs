use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::{
    comm::website::DiscoveryMessage,
    map::{
        check::PotentiallyEq,
        measurement::{Measurement, MeasurementValue},
        upgrade::IsUpgradeable,
    },
    transform::{
        direction::{Direction, DirectionNormalizedVector},
        position::Position,
    },
    utils::map_display::MapDisplay,
};

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub struct Map<const N: usize> {
    cell_discovery_status: [[CellDiscoveryStatus; N]; N],
    // [x, y] corresponds to the walls in the pos-directions from the cell [x, y]
    // .0 is the wall in pos-x, .1 is the wall in pos-y
    wall_discovery_status: [[(WallDiscoveryStatus, WallDiscoveryStatus); N]; N],
}

// Wrapping a Map, nothing special, but signifies, that the Map does not represent a full state,
// but just what is known (--> Every Status is upgradable (e.g. Undiscovered could in truth be
// Discovered))
#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub struct PartialMap<const N: usize>(pub Map<N>);

#[derive(Copy, Clone, PartialEq, Debug, Eq, Default, Serialize, Deserialize, Hash)]
pub enum CellDiscoveryStatus {
    #[default]
    Undiscovered, //No information about the Cell whatsoever
    Discovered, //Sensor reached into cell, but mouse was not physically within its bounds
    Visited,    //Mouse was within cell
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Default, Serialize, Deserialize, Hash)]
pub enum WallDiscoveryStatus {
    #[default]
    Undiscovered,
    Exists(bool),
    Visited, // Like Exists(false), but actually drove over it
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct WallDiscovery {
    pub new_status: WallDiscoveryStatus,
    pub from_cell: Position,
    pub in_direction: Direction,
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CellDiscovery {
    pub new_status: CellDiscoveryStatus,
    pub at_cell: Position,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapInconsistencyError {
    OutsideBounds { x: i64, y: i64 },
    // New Measurement conflicts with prev. measurement. --> Walls in the direction of
    // Measurement-direction are different than previously assumed
    Conflicting(Measurement, Vec<Position>),
}

impl<const N: usize> Map<N> {
    pub fn new() -> Self {
        let cell_discovery_status = [[CellDiscoveryStatus::default(); N]; N];
        let mut wall_discovery_status = [
            //Column
            [(
            WallDiscoveryStatus::default(),
            WallDiscoveryStatus::default(),
        ); N]; N];

        let dim = N;

        // Setting the entire bottom (pos y)'s pos-y-wall to exist (bounds)
        for x in 0..N {
            wall_discovery_status[x][dim - 1].1 = WallDiscoveryStatus::Exists(true);
        }

        // Setting the entire right (pos x)'s pos-x-wall to exist (bounds)
        for y in 0..N {
            wall_discovery_status[dim - 1][y].0 = WallDiscoveryStatus::Exists(true);
        }

        Map {
            cell_discovery_status,
            wall_discovery_status,
        }
    }

    pub fn is_fully_discovered(&self) -> bool {
        for col in self.cell_discovery_status.iter() {
            for cell in col.iter() {
                if *cell == CellDiscoveryStatus::Undiscovered {
                    return false;
                }
            }
        }
        for col in self.wall_discovery_status.iter() {
            for (w_0, w_1) in col.iter() {
                if *w_0 == WallDiscoveryStatus::Undiscovered
                    || *w_1 == WallDiscoveryStatus::Undiscovered
                {
                    return false;
                }
            }
        }

        true
    }

    pub fn wall(
        &self,
        position: &Position,
        in_direction: &Direction,
    ) -> Option<&WallDiscoveryStatus> {
        let x: usize = position.x.try_into().expect("POSITION is u32 to be const size across devices, number should be well in bounds for usize-conversion though");

        let y: usize = position.y.try_into().expect("POSITION is u32 to be const size across devices, number should be well in bounds for usize-conversion though");

        if x >= N || y >= N {
            return None;
        }
        match in_direction {
            Direction::PosX => Some(&self.wall_discovery_status[x][y].0),
            Direction::PosY => Some(&self.wall_discovery_status[x][y].1),
            Direction::NegX => {
                if position.x > 0 {
                    Some(&self.wall_discovery_status[x - 1][y].0)
                } else {
                    None
                }
            }
            Direction::NegY => {
                if position.y > 0 {
                    Some(&self.wall_discovery_status[x][y - 1].1)
                } else {
                    None
                }
            }
        }
    }

    pub fn wall_mut(
        &mut self,
        position: &Position,
        in_direction: &Direction,
    ) -> Option<&mut WallDiscoveryStatus> {
        let x: usize = position.x.try_into().expect("POSITION is u32 to be const size across devices, number should be well in bounds for usize-conversion though");

        let y: usize = position.y.try_into().expect("POSITION is u32 to be const size across devices, number should be well in bounds for usize-conversion though");

        if x >= N || y >= N {
            return None;
        }
        match in_direction {
            Direction::PosX => Some(&mut self.wall_discovery_status[x][y].0),
            Direction::PosY => Some(&mut self.wall_discovery_status[x][y].1),
            Direction::NegX => {
                if position.x > 0 {
                    Some(&mut self.wall_discovery_status[x - 1][y].0)
                } else {
                    None
                }
            }
            Direction::NegY => {
                if position.y > 0 {
                    Some(&mut self.wall_discovery_status[x][y - 1].1)
                } else {
                    None
                }
            }
        }
    }

    pub fn cell(&self, position: &Position) -> Option<&CellDiscoveryStatus> {
        let x: usize = position.x.try_into().expect("POSITION is u32 to be const size across devices, number should be well in bounds for usize-conversion though");

        let y: usize = position.y.try_into().expect("POSITION is u32 to be const size across devices, number should be well in bounds for usize-conversion though");

        if x >= N || y >= N {
            return None;
        }

        Some(&self.cell_discovery_status[x][y])
    }

    pub fn cell_mut(&mut self, position: &Position) -> Option<&mut CellDiscoveryStatus> {
        let x: usize = position.x.try_into().expect("POSITION is u32 to be const size across devices, number should be well in bounds for usize-conversion though");

        let y: usize = position.y.try_into().expect("POSITION is u32 to be const size across devices, number should be well in bounds for usize-conversion though");

        if x >= N || y >= N {
            return None;
        }

        Some(&mut self.cell_discovery_status[x][y])
    }

    // pub fn apply_measurement(
    //     &mut self,
    //     measurement: &Measurement,
    // ) -> Result<DiscoveryMessage, MapInconsistencyError> {
    //     let direction = measurement.direction;
    //     let value = measurement.value;
    //     let from_pos = measurement.position;
    //
    //     debug!(target: "map", "MEASUREMENT {from_pos} -> {direction} | {value}");
    //
    //     let mut wall_discoveries: Vec<WallDiscovery> = vec![];
    //     let mut cell_discoveries: Vec<CellDiscovery> = vec![];
    //
    //     let cell_status = self
    //         .cell_mut(&from_pos)
    //         .ok_or(MapInconsistencyError::OutsideBounds {
    //             x: from_pos.x as i64,
    //             y: from_pos.y as i64,
    //         })?;
    //
    //     if *cell_status != CellDiscoveryStatus::Visited {
    //         debug!(target: "map/discovery", "VISITED {from_pos:?}");
    //         cell_discoveries.push(CellDiscovery {
    //             new_status: CellDiscoveryStatus::Visited,
    //             at_cell: from_pos,
    //         });
    //     }
    //
    //     *cell_status = CellDiscoveryStatus::Visited;
    //
    //     let (no_walls_up_to_depth, hit_wall_at_end) = match value {
    //         crate::map::measurement::MeasurementValue::OutsideRange { at_least_cells } => {
    //             (at_least_cells, false)
    //         }
    //         crate::map::measurement::MeasurementValue::Value { cells } => (cells, true),
    //     };
    //     debug!(target: "map", "MEASUREMENT depth={no_walls_up_to_depth}, hits_wall={hit_wall_at_end}");
    //
    //     let dir_norm_vec: DirectionNormalizedVector = direction.into();
    //     let dx = dir_norm_vec.x as i64;
    //     let dy = dir_norm_vec.y as i64;
    //
    //     let mut inconsistencies = vec![];
    //
    //     // todo!("Update Discoveries");
    //
    //     // INFO: going through all the cells (at least none) that were passed over by the vision-ray
    //
    //     for i in 0..no_walls_up_to_depth as i64 {
    //         debug!(target: "map", "MEASUREMENT Exploring depth = {i}");
    //         let x_offset = dx * i;
    //         let y_offset = dy * i;
    //
    //         let x_pos = from_pos.x as i64 + x_offset;
    //         let y_pos = from_pos.y as i64 + y_offset;
    //
    //         if x_pos < 0 || y_pos < 0 {
    //             error!(target: "map", "MEASUREMENT OUTSIDE BOUNDS (at_step={i}, pos=({x_pos}, {y_pos}))");
    //             return Err(MapInconsistencyError::OutsideBounds { x: x_pos, y: y_pos });
    //         }
    //
    //         let pos = Position {
    //             x: x_pos as u32,
    //             y: y_pos as u32,
    //         };
    //
    //         let cell_status = self
    //             .cell_mut(&pos)
    //             .ok_or(MapInconsistencyError::OutsideBounds {
    //                 x: pos.x as i64,
    //                 y: pos.y as i64,
    //             })?;
    //
    //         if *cell_status == CellDiscoveryStatus::Undiscovered {
    //             // Updated cell status
    //             debug!(target: "map/discovery", "DISCOVERED {pos:?}");
    //             cell_discoveries.push(CellDiscovery {
    //                 new_status: CellDiscoveryStatus::Discovered,
    //                 at_cell: pos,
    //             });
    //             // println!("aksdhfö {:#?}",  pos);
    //             *cell_status = CellDiscoveryStatus::Discovered;
    //         }
    //
    //         if (x_pos == 0 && direction == Direction::NegX)
    //             || (y_pos == 0 && direction == Direction::NegY)
    //         {
    //             if 0 == match measurement.value {
    //                 crate::map::measurement::MeasurementValue::OutsideRange { at_least_cells } => {
    //                     at_least_cells
    //                 }
    //                 crate::map::measurement::MeasurementValue::Value { cells } => cells,
    //             } {
    //                 // Seeing the map-boundaries is valid
    //                 continue;
    //             }
    //         }
    //
    //         let wall = self
    //             .wall_mut(&pos, &direction)
    //             .ok_or(MapInconsistencyError::OutsideBounds { x: x_pos, y: y_pos })?;
    //
    //         if *wall == WallDiscoveryStatus::Exists(true) {
    //             error!(target: "map/discovery", "DISCOVERED INCONSISTENCY {pos:?} {direction:?}");
    //             // SHOULDN'T be true --> conflicts with
    //             // current measurement
    //             inconsistencies.push(pos);
    //         }
    //
    //         // Check whether cell-boundary was visited / discovered before
    //         if *wall != WallDiscoveryStatus::Exists(false) || *wall != WallDiscoveryStatus::Visited
    //         {
    //             debug!(target: "map/discovery", "DISCOVERED {pos:?}{direction:?}");
    //             wall_discoveries.push(WallDiscovery {
    //                 new_status: WallDiscoveryStatus::Exists(false),
    //                 from_cell: pos,
    //                 in_direction: direction,
    //             });
    //         }
    //         if *wall != WallDiscoveryStatus::Visited {
    //             *wall = WallDiscoveryStatus::Exists(false);
    //         }
    //     }
    //
    //     // INFO: Last cell before measurement-end (either reached wall or measurement limit),
    //     // either way: cell was discovered
    //
    //     debug!(target: "map", "Handle last cell");
    //     let x_pos = from_pos.x as i64 + dx * no_walls_up_to_depth as i64;
    //     let y_pos = from_pos.y as i64 + dy * no_walls_up_to_depth as i64;
    //
    //     if x_pos < 0 || y_pos < 0 {
    //         error!(target: "map", "MEASUREMENT OUTSIDE BOUNDS (at_step=END, pos=({x_pos}, {y_pos}))");
    //         return Err(MapInconsistencyError::OutsideBounds { x: x_pos, y: y_pos });
    //     }
    //
    //     let pos = Position {
    //         x: x_pos as u32,
    //         y: y_pos as u32,
    //     };
    //
    //     let cell_status = self
    //         .cell_mut(&pos)
    //         .ok_or(MapInconsistencyError::OutsideBounds {
    //             x: pos.x as i64,
    //             y: pos.y as i64,
    //         })?;
    //
    //     if *cell_status == CellDiscoveryStatus::Undiscovered {
    //         // println!("aksdhfö {:#?}",  pos);
    //         debug!(target: "map/discovery", "DISCOVERED {pos:?}");
    //         cell_discoveries.push(CellDiscovery {
    //             new_status: CellDiscoveryStatus::Discovered,
    //             at_cell: pos,
    //         });
    //         *cell_status = CellDiscoveryStatus::Discovered;
    //     }
    //
    //     if hit_wall_at_end {
    //         // if (x_pos == 0 && direction == Direction::NegX) || (y_pos == 0 && direction == Direction::NegY) {
    //         //     if 0 == match measurement.value {
    //         //         crate::map::measurement::MeasurementValue::OutsideRange { at_least_cells } => at_least_cells,
    //         //         crate::map::measurement::MeasurementValue::Value { cells } => cells,
    //         //     } {
    //         //         // Seeing the map-boundaries is valid
    //         //         break;
    //         //     }
    //         // }
    //         // If it is none, that is ok (the left and top edge return none)
    //         if !(((x_pos == 0 && direction == Direction::NegX)
    //             || (y_pos == 0 && direction == Direction::NegY))
    //             && (measurement.value == MeasurementValue::Value { cells: 0 }
    //                 || measurement.value == MeasurementValue::OutsideRange { at_least_cells: 0 }))
    //         {
    //             if let Some(wall) = self.wall_mut(&pos, &direction) {
    //                 if *wall == WallDiscoveryStatus::Exists(false)
    //                     || *wall == WallDiscoveryStatus::Visited
    //                 {
    //                     // Wall found which was already assumed to not exists
    //                     inconsistencies.push(pos);
    //                 } else if *wall != WallDiscoveryStatus::Exists(true) {
    //                     // Wall found, where we previously assumed something else, though it does not
    //                     // conflict
    //                     debug!(target: "map/discovery", "DISCOVERED {pos:?}{direction:?}");
    //                     wall_discoveries.push(WallDiscovery {
    //                         new_status: WallDiscoveryStatus::Exists(true),
    //                         from_cell: pos,
    //                         in_direction: direction,
    //                     })
    //                 }
    //
    //                 *wall = WallDiscoveryStatus::Exists(true);
    //             }
    //         }
    //     }
    //
    //     if !inconsistencies.is_empty() {
    //         return Err(MapInconsistencyError::Conflicting(
    //             *measurement,
    //             inconsistencies,
    //         ));
    //     }
    //
    //     Ok(DiscoveryMessage {
    //         cell_discoveries,
    //         wall_discoveries,
    //     })
    // }

    pub fn apply_measurement(
        &mut self,
        measurement: &Measurement,
    ) -> Result<DiscoveryMessage, MapInconsistencyError> {
        let direction = measurement.direction;
        let value = measurement.value;
        let from_pos = measurement.position;

        debug!(target: "map", "MEASUREMENT {from_pos} -> {direction} | {value}");

        let mut wall_discoveries: Vec<WallDiscovery> = vec![];
        let mut cell_discoveries: Vec<CellDiscovery> = vec![];
        let mut inconsistencies = vec![];

        let cell_status = self
            .cell_mut(&from_pos)
            .ok_or(MapInconsistencyError::OutsideBounds {
                x: from_pos.x as i64,
                y: from_pos.y as i64,
            })?;

        if *cell_status != CellDiscoveryStatus::Visited {
            debug!(target: "map/discovery", "VISITED {from_pos:?}");
            cell_discoveries.push(CellDiscovery {
                new_status: CellDiscoveryStatus::Visited,
                at_cell: from_pos,
            });
        }

        *cell_status = CellDiscoveryStatus::Visited;

        let (no_walls_up_to_depth, hit_wall_at_end) = match value {
            crate::map::measurement::MeasurementValue::OutsideRange { at_least_cells } => {
                (at_least_cells, false)
            }
            crate::map::measurement::MeasurementValue::Value { cells } => (cells, true),
        };

        debug!(target: "map", "MEASUREMENT depth={no_walls_up_to_depth}, hits_wall={hit_wall_at_end}");

        let dir_norm_vec: DirectionNormalizedVector = direction.into();
        let dx = dir_norm_vec.x as i32;
        let dy = dir_norm_vec.y as i32;

        for depth_sample in 0..=no_walls_up_to_depth {
            let measurement_determined = if depth_sample == no_walls_up_to_depth {
                if hit_wall_at_end {
                    WallDiscoveryStatus::Exists(true)
                } else {
                    WallDiscoveryStatus::Undiscovered
                }
            } else {
                WallDiscoveryStatus::Exists(false)
            };

            let sample_pos_x = from_pos.x as i32 + dx * depth_sample as i32;
            let sample_pos_y = from_pos.y as i32 + dy * depth_sample as i32;

            if sample_pos_x < 0
                || sample_pos_y < 0
                || sample_pos_x as usize + 1 > N
                || sample_pos_y as usize + 1 > N
            {
                // Can only get to this invalid state by going out of bounds
                break;
            }

            let pos = Position {
                x: sample_pos_x as u32,
                y: sample_pos_y as u32,
            };

            let sample_cell = self.cell_mut(&pos).expect("Already did bounds-check");
            if CellDiscoveryStatus::Discovered.could_be_upgrade_of(sample_cell) {
                // We can upgrade the cell (Discovered can be seen as an upgrade of it)
                *sample_cell = CellDiscoveryStatus::Discovered;
                cell_discoveries.push(CellDiscovery {
                    new_status: CellDiscoveryStatus::Discovered,
                    at_cell: pos,
                })
            }

            let Some(sample_wall) = self.wall_mut(&pos, &direction) else {
                if ((pos.x == 0 && direction == Direction::NegX)
                    || (pos.y == 0 && direction == Direction::NegY))
                    && measurement_determined.potentially_eq(&WallDiscoveryStatus::Exists(true))
                {
                    // Is allowed, outside way just has to stay potentially_eq(Exists(true))
                    return Err(MapInconsistencyError::OutsideBounds {
                        x: pos.x as _,
                        y: pos.y as _,
                    });
                } else {
                    inconsistencies.push(pos);
                    // return Err(MapInconsistencyError::Conflicting(*measurement, )
                }
                break;
            };
            if measurement_determined.could_be_upgrade_of(sample_wall) {
                // We can upgrade the sample_wall to measurement_determined
                *sample_wall = measurement_determined;
                wall_discoveries.push(WallDiscovery {
                    new_status: measurement_determined,
                    from_cell: pos,
                    in_direction: direction,
                });
            } else {
                inconsistencies.push(pos);
            }
        }

        if inconsistencies.is_empty() {
            Ok(DiscoveryMessage {
                cell_discoveries,
                wall_discoveries,
            })
        } else {
            Err(MapInconsistencyError::Conflicting(
                *measurement,
                inconsistencies,
            ))
        }
    }

    pub fn is_pos_map_boundary(&self, position: &Position, direction: &Direction) -> bool {
        (position.x == 0 && *direction == Direction::NegX)
            || (position.y == 0 && *direction == Direction::NegY)
    }
}

impl<const N: usize> Default for Map<N> {
    fn default() -> Self {
        Map::new()
    }
}

impl<const N: usize> Display for Map<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug!(target: "tests/map/display", "Starting format");

        let display = MapDisplay::from(self);

        writeln!(f, "{}", display)?;

        Ok(())
    }
}

impl<const N: usize> Deref for PartialMap<N> {
    type Target = Map<N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const N: usize> DerefMut for PartialMap<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> From<Map<N>> for PartialMap<N> {
    fn from(value: Map<N>) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<PartialMap<N>> for Map<N> {
    fn from(value: PartialMap<N>) -> Self {
        value.0
    }
}
