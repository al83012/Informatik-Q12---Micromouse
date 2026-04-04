use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::direction::Direction;

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
        if x > 0 && y > 0 {
            Some(Position {
                x: x as u32,
                y: y as u32,
            })
        } else {
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
            pos: (self.pos + self.dir.clone().steps_in_dir(fwd_steps))?,
            dir: self.dir,
        })
    }
}
