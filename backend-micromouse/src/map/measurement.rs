use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    transform::direction::Direction,
    transform::position::Position,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MeasurementValue {
    // at_least_cells = 0 --> Means, that there is no information at all (Can guarantee, that there
    // is space, at least to the next potential cell-wall)
    OutsideRange { at_least_cells: u32 },
    // cells = 0 --> Means, that there is a wall at the next location (cannot go in that direction
    // from the current cell without collision)
    Value { cells: u32 },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Measurement {
    pub value: MeasurementValue,
    pub direction: Direction,
    pub position: Position,
}

impl Display for MeasurementValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self {
                MeasurementValue::OutsideRange { at_least_cells } => at_least_cells,
                MeasurementValue::Value { cells } => cells,
            },
            if let MeasurementValue::Value { cells: _ } = self {
                "|"
            } else {
                "+"
            }
        )
    }
}
