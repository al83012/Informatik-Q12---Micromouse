use crate::{direction::Direction, position::Position};


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MeasurementValue{
    OutsideRange{at_least_cells: usize},
    Value{cells: usize}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct  Measurement {
    value: MeasurementValue,
    direction: Direction,
    position: Position,
}


