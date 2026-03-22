use serde::{Deserialize, Serialize};

use crate::map::{CellDiscovery, WallDiscovery};


#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryMessage{
    pub cell_discoveries: Vec<CellDiscovery>,
    pub wall_discoveries: Vec<WallDiscovery>
}
