use std::collections::{HashSet, VecDeque};

use crate::{
    comm::micromouse_message::MovementType,
    map::world_data::WorldData,
    transform::{direction::Direction, position::MouseTransform},
    utils::path::Path,
};

pub struct DepthFirstWithCurrent(DepthFirstBase);

pub struct DepthFirstBase {
    intersection_stack: VecDeque<Intersection>,
    path_from_start: Path,
    map_size: usize,
}

pub struct Intersection {
    visitable_directions: HashSet<Direction>,
}

pub enum IntersectionOrdering {
    Undefined,
    LowestMoves,
    LowestCost { turn_value: u32, move_value: u32 },
}

impl DepthFirstBase {
    pub fn with_current_world<const N: usize>(
        world: impl Into<WorldData<N>>,
    ) -> Option<DepthFirstWithCurrent> {
        todo!("Add the current pos to the path and update the intersection stack if there are any new intersections; Reject if there are not enough measures for the step to be fully complete")
    }
}

impl DepthFirstWithCurrent {
    pub fn moves_to_next_intersection(
        &mut self,
        intersection_ordering: IntersectionOrdering,
    ) -> (Vec<MovementType>, MouseTransform) {
        todo!("Plot the path back to the next valid intersection (might just mean turning around); Return the transform after these moves are applied")
        // Also: skips any intersections that are guaranteed to have 0 max_steps
    }

    pub fn max_steps_in_direction(&self, from_origin: MouseTransform) {
        todo!("Measure in direction until reaching known wall or visited")
    }

    // Prepares for next step / successor
    pub fn finish_step(self) -> DepthFirstBase {
        self.0
    }
}
