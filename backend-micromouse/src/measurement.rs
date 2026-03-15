use crate::{direction::Direction, position::Position};


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MeasurementValue{
    OutsideRange{at_least_cells: u32},
    Value{cells: u32}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct  Measurement {
    pub value: MeasurementValue,
    pub direction: Direction,
    pub position: Position,
}



