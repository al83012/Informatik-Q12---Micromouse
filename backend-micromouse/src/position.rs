use std::ops::Deref;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{comm::micromouse_message::MovementType, direction::{Direction, DirectionNormalizedVector}};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionOffset {
    pub d_x: i32,
    pub d_y: i32,
}

impl From<DirectionNormalizedVector> for PositionOffset {
    fn from(value: DirectionNormalizedVector) -> Self {
        Self {
            d_x: value.x as i32,
            d_y: value.y as i32
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MouseTransform {
    pub pos: Position,
    pub dir: Direction,
}

impl Default for MouseTransform {
    fn default() -> Self {
        Self {
            pos: Position { x: 0, y: 0 },
            dir: Direction::PosX,
        }
    }
}

impl std::ops::Add<PositionOffset> for Position {
    type Output = Option<Position>;
    fn add(self, rhs: PositionOffset) -> Self::Output {
        let x = self.x as i32 + rhs.d_x;
        let y = self.y as i32 + rhs.d_y;
        if x >= 0 && y >= 0 {
            Some(Position {
                x: x as u32,
                y: y as u32,
            })
        } else {
            warn!(target: "tests/op", "Addition underflowed: ({x}, {y})");
            None
        }
    }
}

impl MouseTransform {
    pub fn rotated(self, intervals_counter_clockwise: i8) -> Self {
        MouseTransform {
            pos: self.pos,
            dir: self.dir.rotated(intervals_counter_clockwise),
        }
    }
    pub fn moved(self, fwd_steps: u8) -> Option<Self> {
        Some(MouseTransform {
            pos: (self.pos + self.dir.steps_in_dir(fwd_steps))?,
            dir: self.dir,
        })
    }
    pub fn step_once(self, movement: MovementType) -> Option<Self> {
        match movement {
            MovementType::Turn(i) => Some(self.rotated(i.signum())),
            MovementType::Move(_) => self.moved(1)
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
