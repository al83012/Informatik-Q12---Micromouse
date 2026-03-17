use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    comm::website::DiscoveryMessage, direction::{Direction, DirectionNormalizedVector}, measurement::Measurement, position::Position
};

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub struct Map<const N: usize> {
    cell_discovery_status: [[CellDiscoveryStatus; N]; N],
    // [x, y] corresponds to the walls in the pos-directions from the cell [x, y]
    // .0 is the wall in pos-x, .1 is the wall in pos-y
    wall_discovery_status: [[(WallDiscoveryStatus, WallDiscoveryStatus); N]; N],
}

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
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallDiscovery {
    pub new_status: WallDiscoveryStatus,
    pub from_cell: Position,
    pub in_direction: Direction,
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellDiscovery {
    pub new_status: CellDiscoveryStatus,
    pub at_cell: Position
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

    pub fn update_discovery(
        &mut self,
        measurement: &Measurement,
    ) ->  Result<DiscoveryMessage, MapInconsistencyError> {
        let direction = measurement.direction;
        let value = measurement.value;
        let from_pos = measurement.position;

        let cell_status = self
            .cell_mut(&from_pos)
            .ok_or(MapInconsistencyError::OutsideBounds {
                x: from_pos.x as i64,
                y: from_pos.y as i64,
            })?;

        *cell_status = CellDiscoveryStatus::Visited;

        let (no_walls_up_to_depth, hit_wall_at_end) = match value {
            crate::measurement::MeasurementValue::OutsideRange { at_least_cells } => {
                (at_least_cells, false)
            }
            crate::measurement::MeasurementValue::Value { cells } => (cells, true),
        };

        let dir_norm_vec: DirectionNormalizedVector = direction.into();
        let dx = dir_norm_vec.x as i64;
        let dy = dir_norm_vec.y as i64;

        let mut inconsistencies = vec![];

        let mut wall_discoveries: Vec<WallDiscovery> = vec![];
        let mut cell_discoveries: Vec<CellDiscovery> = vec![];

        todo!("Update Discoveries");



        // INFO: going through all the cells (at least none) that were passed over by the vision-ray

        for i in 0..no_walls_up_to_depth as i64 {
            let x_offset = dx * i;
            let y_offset = dy * i;

            let x_pos = from_pos.x as i64 + x_offset;
            let y_pos = from_pos.y as i64 + y_offset;

            if x_pos < 0 || y_pos < 0 {
                return Err(MapInconsistencyError::OutsideBounds { x: x_pos, y: y_pos });
            }

            let pos = Position {
                x: x_pos as u32,
                y: y_pos as u32,
            };

            let cell_status = self
                .cell_mut(&pos)
                .ok_or(MapInconsistencyError::OutsideBounds {
                    x: pos.x as i64,
                    y: pos.y as i64,
                })?;

            if *cell_status == CellDiscoveryStatus::Undiscovered {
                // println!("aksdhfö {:#?}",  pos);
                *cell_status = CellDiscoveryStatus::Discovered;
            }

            let wall = self
                .wall_mut(&pos, &direction)
                .ok_or(MapInconsistencyError::OutsideBounds { x: x_pos, y: y_pos })?;

            if *wall == WallDiscoveryStatus::Exists(true) {
                // SHOULDN'T be true --> conflicts with
                // current measurement
                inconsistencies.push(pos);
            }
            *wall = WallDiscoveryStatus::Exists(false);
        }



        // INFO: Last cell before measurement-end (either reached wall or measurement limit),
        // either way: cell was discovered

        let x_pos = from_pos.x as i64 + dx * no_walls_up_to_depth as i64;
        let y_pos = from_pos.y as i64 + dy * no_walls_up_to_depth as i64;

        if x_pos < 0 || y_pos < 0 {
            return Err(MapInconsistencyError::OutsideBounds { x: x_pos, y: y_pos });
        }

        let pos = Position {
            x: x_pos as u32,
            y: y_pos as u32,
        };

        let cell_status = self
            .cell_mut(&pos)
            .ok_or(MapInconsistencyError::OutsideBounds {
                x: pos.x as i64,
                y: pos.y as i64,
            })?;

        if *cell_status == CellDiscoveryStatus::Undiscovered {
            // println!("aksdhfö {:#?}",  pos);
            *cell_status = CellDiscoveryStatus::Discovered;
        }

        if hit_wall_at_end {
            let wall = self
                .wall_mut(&pos, &direction)
                .ok_or(MapInconsistencyError::OutsideBounds { x: x_pos, y: y_pos })?;

            if *wall == WallDiscoveryStatus::Exists(false) {
                inconsistencies.push(pos);
            }
            *wall = WallDiscoveryStatus::Exists(true);
        }

        if !inconsistencies.is_empty() {
            return Err(MapInconsistencyError::Conflicting(
                *measurement,
                inconsistencies,
            ));
        }

        todo!("Ok(())")
    }
}

impl<const N: usize> Default for Map<N> {
    fn default() -> Self {
        Map::new()
    }
}

impl<const N: usize> Display for Map<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dim = self.cell_discovery_status.len();
        let mut visuals = vec![vec![' '; (dim * 4) + 1]; (dim * 2) + 1];

        for x in 0..dim + 1 {
            for y in 0..dim + 1 {
                visuals[y * 2][x * 4] = '+';
            }
        }

        for x in 0..dim {
            let start = x * 4 + 1;
            for x_i in 0..3 {
                visuals[0][x_i + start] = '-';
            }
        }
        for y in 0..dim {
            visuals[y * 2 + 1][0] = '|';
        }

        for x in 0..dim as u32 {
            for y in 0..dim as u32 {
                let pos = Position { x, y };
                let wall_right = self.wall(&pos, &Direction::PosX).unwrap();
                let wall_down = self.wall(&pos, &Direction::PosY).unwrap();

                visuals[y as usize * 2 + 1][(x as usize + 1) * 4] = match wall_right {
                    WallDiscoveryStatus::Undiscovered => '?',
                    WallDiscoveryStatus::Exists(true) => '|',
                    _ => ' ',
                };

                match wall_down {
                    WallDiscoveryStatus::Undiscovered => {
                        visuals[(y as usize + 1) * 2][x as usize * 4 + 2] = '?';
                    }
                    WallDiscoveryStatus::Exists(true) => {
                        let start = x as usize * 4 + 1;
                        for x_i in 0..3 {
                            visuals[(y as usize + 1) * 2][x_i + start] = '-';
                        }
                    }
                    _ => {}
                }

                let cell_discovery_status = self.cell(&pos).unwrap();
                visuals[(y as usize) * 2 + 1][(x as usize) * 4 + 2] = match cell_discovery_status {
                    CellDiscoveryStatus::Undiscovered => ' ',
                    CellDiscoveryStatus::Discovered => '·',
                    CellDiscoveryStatus::Visited => '□',
                };
            }
        }

        for line in visuals {
            for c in line {
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
