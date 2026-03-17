use serde::{Deserialize, Serialize};



#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position{pub x: u32, pub y: u32}
