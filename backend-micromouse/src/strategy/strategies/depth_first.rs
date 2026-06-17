use std::{
    collections::{HashMap, HashSet},
    usize,
};

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    comm::micromouse_message::{
        Command, InterruptAction, InterruptStep, MeasurementInterrupt, MovementType,
        TransformedMovement,
    },
    map::{
        map::{CellDiscoveryStatus, Map, PartialMap, WallDiscoveryStatus},
        world_data::{PartialWorldData, WorldData},
    },
    strategy::{
        strategies::utils::{depth_first_base::DepthFirstBase, value_map::ValueMap},
        strategy::{
            ComputedAction, ComputedActions, FromConfig, Strategy, StrategyComputationResult,
            StrategyEndState,
        },
    },
    transform::{
        direction::{Direction, DirectionNormalizedVector, RelativeDirection},
        position::{MouseTransform, Position, RayIterator},
    },
    utils::{
        nonempty::{NonEmpty, PotentiallyNonEmpty},
        path::Path,
    },
};

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct DepthFirstConfig {
    pub forward_first: bool,
}

#[derive(Clone, Debug)]
pub struct DepthFirst<const N: usize> {
    df_base: DepthFirstBase,
}

impl<const N: usize> FromConfig<N> for DepthFirst<N> {
    type Config = DepthFirstConfig;

    #[instrument(
        name = "from_config DepthFirst",
        fields(description = "Create new DepthFirst-Strategy instance based on config")
    )]
    fn from_config(config: &Self::Config, starting_state: &WorldData<N>) -> Self {
        todo!("")
    }
}

impl<const N: usize> Strategy<N> for DepthFirst<N> {
    #[instrument(
        name = "next_cmd DepthFirst",
        fields(description = "Try to get next action")
    )]
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,

        // The goal is only important for determining, whether the mouse has reached its goal (in
        // this strategy)
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        if world.mouse.pos == goal.0 {
            return StrategyComputationResult::Computed(Err(StrategyEndState::ReachedGoal));
        }
        todo!()
    }
}
