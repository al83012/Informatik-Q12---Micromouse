use serde::{Deserialize, Serialize};

use crate::strategy::strategy::{FromConfig, Strategy};

#[derive(Clone, Debug)]
pub struct BreadthFirst<const N: usize>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BreadthFirstConfig;

impl<const N: usize> FromConfig<N> for BreadthFirst<N> {
    type Config = BreadthFirstConfig;

    fn from_config(
        config: &Self::Config,
        starting_state: &crate::map::world_data::WorldData<N>,
    ) -> Self {
        todo!()
    }
}

impl<const N: usize> Strategy<N> for BreadthFirst<N> {
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        todo!()
    }
}
