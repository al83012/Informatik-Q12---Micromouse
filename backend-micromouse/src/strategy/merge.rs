
use crate::{
    comm::micromouse_message::Command,
    map::command_world_state::LazyFilteredCommandApplication,
};

pub struct MergeResult {
    pub common: Command,

    // If the merge isn't perfect, the given command applications will not be fully represented
    // just by their common part:
    // Thus, depending on the commands and their world, there may be rests
    pub rest_a: Option<Command>,
    pub rest_b: Option<Command>,
}

pub trait MergeStrategy {
    /// Try to find another command, which will ensure that given the respective worlds of a and b,
    /// it will return exactly the same Outcomes as them
    /// Using LazyFilteredCommandApplication, since it is not always necessary for the
    /// CommandApplication to actually be computed for merging
    fn merge<const N: usize>(
        &self,
        a: &LazyFilteredCommandApplication<N>,
        b: &LazyFilteredCommandApplication<N>,
    ) -> Option<Command>;
}
