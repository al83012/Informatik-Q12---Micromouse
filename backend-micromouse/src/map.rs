use crate::{measurement::Measurement, position::Position};

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub struct Map<const N: usize> {
    cell_discovery_status: [[CellDiscoveryStatus; N]; N],
    wall_discovery_status: [[WallDiscoveryStatus; N]; N],
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
    OutsideBounds(Position),
    // New Measurement conflicts with prev. measurement. --> Walls in the direction of
    // Measurement-direction are different than previously assumed
    Conflicting(Measurement, Vec<Position>),
}

impl<const N: usize> Map<N> {
    pub fn new() -> Self {
        let cell_discovery_status = [[CellDiscoveryStatus::default(); N]; N];
        let wall_discovery_status = [[WallDiscoveryStatus::default(); N]; N];

        Map {
            cell_discovery_status,
            wall_discovery_status,
        }
    }

    pub fn update_discovery(
        &mut self,
        measurement: Measurement,
    ) -> Result<(), MapInconsistencyError> {
        todo!("Check into direction of measurement");
        todo!("Detect inconsistencies");
        Ok(())
    }
}

impl<const N: usize> Default for Map<N> {
    fn default() -> Self {
        Map::new()
    }
}
