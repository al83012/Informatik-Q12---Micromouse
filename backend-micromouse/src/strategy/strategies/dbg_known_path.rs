use serde::{Deserialize, Serialize};

use crate::strategy::strategies::flood_fill::FloodFillConfig;
use crate::utils::nonempty::PotentiallyNonEmpty;
use crate::{
    map::world_data::{PartialWorldData, WorldData},
    strategy::{
        strategies::{
            flood_fill::{FFCost, FloodFill},
            utils::capsule,
        },
        strategy::{
            ComputedAction, ComputedActions, FromConfig, Strategy, StrategyComputationResult,
            WithGraftingFilter,
        },
        strategy_tree::GraftingFilter,
    },
    utils::nonempty::NonEmpty,
};

#[derive(Clone, Debug)]
pub struct DbgKnownPath<const N: usize> {
    ff_strat: FloodFill<N>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DbgKnownPathConfig {
    pub rotation_cost: FFCost,
    pub move_cost: FFCost,
}

impl<const N: usize> FromConfig<N> for DbgKnownPath<N> {
    type Config = DbgKnownPathConfig;

    fn from_config(
        config: &Self::Config,
        starting_state: &crate::map::world_data::WorldData<N>,
    ) -> Self {
        Self {
            ff_strat: FloodFill::from_config(
                &FloodFillConfig {
                    move_cost: config.move_cost.clone(),
                    rotation_cost: config.rotation_cost.clone(),
                    exploration_incentive: 0,
                },
                starting_state,
            ),
        }
    }
}

impl WithGraftingFilter for DbgKnownPathConfig {
    fn require_grafting_filter(&self) -> crate::strategy::strategy_tree::GraftingFilter {
        GraftingFilter::None
    }
}

impl<const N: usize> Strategy<N> for DbgKnownPath<N> {
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        let capsule_map = capsule::as_capsule_map(&world.map);

        let capsule_world = PartialWorldData::<N>::from(WorldData {
            map: capsule_map,
            mouse: world.mouse,
        });

        let res = self.ff_strat.next_cmd(&capsule_world, goal);
        match res {
            StrategyComputationResult::NotEnoughInformation => {
                StrategyComputationResult::NotEnoughInformation
            }
            StrategyComputationResult::Computed(Ok(ComputedActions(actions))) => {
                let actions: Vec<ComputedAction<N, Self>> = actions
                    .into_inner()
                    .into_iter()
                    .map(|a| ComputedAction {
                        after_command: a.after_command,
                        next_strategy_state: a.next_strategy_state.map(|s| Self { ff_strat: s }),
                    })
                    .collect();
                StrategyComputationResult::Computed(Ok(ComputedActions(
                    actions.non_empty().expect("Actions is non-empty"),
                )))
            }
            StrategyComputationResult::Computed(Err(end)) => {
                StrategyComputationResult::Computed(Err(end))
            }
        }
    }
}
