use serde::{Deserialize, Serialize};

use crate::transform::position::PositionOffset;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    PosX,
    PosY,
    NegX,
    NegY,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionAngle(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectionNormalizedVector {
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelativeDirection {
    Forward,
    Left,
    Right,
}

impl std::fmt::Display for RelativeDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RelativeDirection::Forward => "F",
                RelativeDirection::Left => "L",
                RelativeDirection::Right => "R",
            }
        )
    }
}

impl From<Direction> for DirectionAngle {
    fn from(value: Direction) -> Self {
        // Similar to Unit-circle, but y-Axis is pointing down (As the positive Axis is generally
        // downward when working with 2D arrays)
        match value {
            Direction::PosX => Self(0.0),
            Direction::PosY => Self(270.0),
            Direction::NegX => Self(180.0),
            Direction::NegY => Self(90.0),
        }
    }
}

impl From<Direction> for DirectionNormalizedVector {
    fn from(value: Direction) -> Self {
        match value {
            Direction::PosX => Self { x: 1, y: 0 },
            Direction::PosY => Self { x: 0, y: 1 },
            Direction::NegX => Self { x: -1, y: 0 },
            Direction::NegY => Self { x: 0, y: -1 },
        }
    }
}

impl RelativeDirection {
    pub fn transform_by(&self, forward_direction: &Direction) -> Direction {
        match self {
            RelativeDirection::Forward => *forward_direction,
            RelativeDirection::Left => match forward_direction {
                Direction::PosX => Direction::NegY,
                Direction::PosY => Direction::PosX,
                Direction::NegX => Direction::PosY,
                Direction::NegY => Direction::NegX,
            },
            RelativeDirection::Right => match forward_direction {
                Direction::PosX => Direction::PosY,
                Direction::PosY => Direction::NegX,
                Direction::NegX => Direction::NegY,
                Direction::NegY => Direction::PosX,
            },
        }
    }
}

impl Direction {
    pub const COUNTER_CLOCKWISE_ROTATIONS: [Self; 4] = [
        Direction::PosX,
        Direction::NegY,
        Direction::NegX,
        Direction::PosY,
    ];

    fn rotation_index(&self) -> usize {
        for i in 0..4 {
            if &Self::COUNTER_CLOCKWISE_ROTATIONS[i] == self {
                return i;
            }
        }
        panic!("Direction has to be in list");
    }

    //TODO: unittests
    pub fn rotated(&self, interval_counter_clockwise: i8) -> Self {
        let rot_idx = self.rotation_index();
        let transformed_rot_idx = rot_idx as i8 + interval_counter_clockwise;
        let idx = if transformed_rot_idx >= 0 {
            transformed_rot_idx as usize % 4
        } else {
            let from_last =  (transformed_rot_idx.abs() - 1) % 4;
            3 - from_last as usize 
        };
        // let idx = if transformed_rot_idx > 0 {
        //     transformed_rot_idx as usize % 4
        // } else {
        //     let from_last = transformed_rot_idx.abs() as usize % 4;
        //     3 - from_last
        // };

        Self::COUNTER_CLOCKWISE_ROTATIONS[idx]
    }
    pub fn steps_in_dir(self, steps: u8) -> PositionOffset {
        let dir_vec: DirectionNormalizedVector = self.into();
        let d_x = dir_vec.x;
        let d_y = dir_vec.y;
        PositionOffset {
            d_x: d_x as i32 * steps as i32,
            d_y: d_y as i32 * steps as i32,
        }
    }
}



impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Direction::PosX => "+X",
            Direction::PosY => "+Y",
            Direction::NegX => "-X",
            Direction::NegY => "-Y",
        })
    }
}
