use crate::{
    comm::micromouse_message::Command, map::world_data::{PartialWorldData, WorldData},
    transform::position::Position,
};

pub struct GoalPosition(pub Position);
pub trait FromConfig {
    type Config;
    fn from_config(config: Self::Config) -> Self;
}

pub trait Strategy<const N: usize>: FromConfig + Sized {
    fn next_cmd_from_partial(world: PartialWorldData<N>) -> Option<Command>;
    fn next_cmd(world: WorldData<N>) -> Command;
}
