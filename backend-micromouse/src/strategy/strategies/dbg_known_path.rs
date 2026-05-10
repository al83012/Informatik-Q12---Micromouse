
use serde::{Deserialize, Serialize};

use crate::strategy::strategy::{FromConfig, Strategy};

#[derive(Clone, Debug)]
pub struct DbgKnownPath<const N: usize>;

#[derive(Clone, Debug, Deserialize)]
pub struct DbgKnownPathConfig;

impl<const N: usize> FromConfig<N> for DbgKnownPath<N> {
    type Config = DbgKnownPathConfig;

    fn from_config(
        config: &Self::Config,
        starting_state: &crate::map::world_data::WorldData<N>,
    ) -> Self {
        todo!()
    }
}

impl<const N: usize> Strategy<N> for DbgKnownPath<N> {
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        todo!()
    }
}
