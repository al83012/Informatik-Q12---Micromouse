#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    PosX,
    PosY,
    NegX,
    NegY,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionAngle(pub f32);



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectionNormalizedVector{pub x: i8,pub y: i8}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelativeDirection {
    Forward,
    Left,
    Right,
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
            Direction::PosX => Self{x: 1, y: 0},
            Direction::PosY => Self{x: 0, y: 1},
            Direction::NegX => Self{x: -1, y: 0},
            Direction::NegY => Self{x: 0, y: -1},
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
