use crate::{
    comm::micromouse::FormatError, direction::{Direction, RelativeDirection}, position::Position
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MeasurementValue {
    // at_least_cells = 0 --> Means, that there is no information at all (Can guarantee, that there
    // is space, at least to the next potential cell-wall)
    OutsideRange { at_least_cells: u32 },
    // cells = 0 --> Means, that there is a wall at the next location (cannot go in that direction
    // from the current cell without collision)
    Value { cells: u32 },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Measurement {
    pub value: MeasurementValue,
    pub direction: Direction,
    pub position: Position,
}

impl TryFrom<String> for RelativeDirection {
    type Error = FormatError<Self>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "L" => Ok(Self::Left),
            "R" => Ok(Self::Right),
            "F" => Ok(Self::Forward),
            _ => Err(FormatError::new(value))
        }
    }
}
