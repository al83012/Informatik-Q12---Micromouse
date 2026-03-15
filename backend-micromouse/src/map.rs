use crate::{
    direction::{Direction, DirectionNormalizedVector},
    measurement::Measurement,
    position::Position,
};

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub struct Map<const N: usize> {
    cell_discovery_status: [[CellDiscoveryStatus; N]; N],
    // [x, y] corresponds to the walls in the pos-directions from the cell [x, y]
    // .0 is the wall in pos-x, .1 is the wall in pos-y
    wall_discovery_status: [[(WallDiscoveryStatus, WallDiscoveryStatus); N]; N],
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Default)]
pub enum CellDiscoveryStatus {
    #[default]
    Undiscovered, //No information about the Cell whatsoever
    Discovered, //Sensor reached into cell, but mouse was not physically within its bounds
    Visited,    //Mouse was within cell
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Default)]
pub enum WallDiscoveryStatus {
    #[default]
    Undiscovered,
    Exists(bool),
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
        let mut wall_discovery_status = [[(
            WallDiscoveryStatus::default(),
            WallDiscoveryStatus::default(),
        ); N]; N];

        let dim = N;

        // Setting the entire bottom (pos y)'s pos-y-wall to exist (bounds)
        for x in 0..N {
            wall_discovery_status[x][dim - 1].1 = WallDiscoveryStatus::Exists(true);
        }

        // Setting the entire right (pos x)'s pos-x-wall to exist (bounds)
        for y in 0..(N - 1) {
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
                    Some(&self.wall_discovery_status[x][y - 1].0)
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
                    Some(&mut self.wall_discovery_status[x][y - 1].0)
                } else {
                    None
                }
            }
        }
    }

    pub fn update_discovery(
        &mut self,
        measurement: &Measurement,
    ) -> Result<(), MapInconsistencyError> {
        let direction = measurement.direction;
        let value = measurement.value;
        let from_pos = measurement.position;

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

        if hit_wall_at_end {
            let x_pos = from_pos.x as i64 + dx * no_walls_up_to_depth as i64;
            let y_pos = from_pos.y as i64 + dy * no_walls_up_to_depth as i64;

            if x_pos < 0 || y_pos < 0 {
                return Err(MapInconsistencyError::OutsideBounds { x: x_pos, y: y_pos });
            }

            let pos = Position {
                x: x_pos as u32,
                y: y_pos as u32,
            };

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

        Ok(())
    }
}

impl<const N: usize> Default for Map<N> {
    fn default() -> Self {
        Map::new()
    }
}
