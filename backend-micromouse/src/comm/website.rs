use serde::{Deserialize, Serialize};

use crate::{map::map::{CellDiscovery, WallDiscovery}, utils::nonempty::{NonEmpty, PotentiallyNonEmpty}};


#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct DiscoveryMessage{
    pub cell_discoveries: Vec<CellDiscovery>,
    pub wall_discoveries: Vec<WallDiscovery>
}


impl PotentiallyNonEmpty for  DiscoveryMessage {
    fn is_empty(&self) -> bool {
        self.cell_discoveries.is_empty() && self.wall_discoveries.is_empty()
    }
}
