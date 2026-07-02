
use serde::{Deserialize, Serialize};

use crate::strategy::{strategy::{FromConfig, Strategy}, strategy_tree::GraftingFilter};

#[derive(Clone, Debug)]
pub struct FloodFill<const N: usize>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FloodFillConfig;

impl<const N: usize> FromConfig<N> for FloodFill<N> {
    type Config = FloodFillConfig;

    fn from_config(
        config: &Self::Config,
        starting_state: &crate::map::world_data::WorldData<N>,
    ) -> Self {
        todo!()
    }
    fn require_grafting_filter(&self) -> crate::strategy::strategy_tree::GraftingFilter {
        GraftingFilter::None
    }
}


impl<const N: usize> Strategy<N> for FloodFill<N> {
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        todo!()
    }
}
