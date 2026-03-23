use serde::{Deserialize, Serialize};

use crate::direction::Direction;



#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position{pub x: u32, pub y: u32}


pub struct MouseTransform {
    pub pos: Position,
    pub dir: Direction
}
