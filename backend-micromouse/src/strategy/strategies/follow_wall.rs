
use serde::{Deserialize, Serialize};

use crate::strategy::strategy::{FromConfig, Strategy};

#[derive(Clone, Debug)]
pub struct FollowWall<const N: usize>;

#[derive(Clone, Debug, Deserialize)]
pub struct FollowWallConfig;

impl<const N: usize> FromConfig<N> for FollowWall<N> {
    type Config = FollowWallConfig;

    fn from_config(
        config: &Self::Config,
        starting_state: &crate::map::world_data::WorldData<N>,
    ) -> Self {
        todo!()
    }
}


impl<const N: usize> Strategy<N> for FollowWall<N> {
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        todo!()
    }
}
