use serde::{Deserialize, Serialize};

use crate::map::{CellDiscovery, WallDiscovery};


#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryMessage{
    cell_discoveries: Vec<CellDiscovery>,
    wall_discoveries: Vec<WallDiscovery>
}
