use serde::{Deserialize, Serialize};
use tracing::info;

use crate::comm::micromouse_message::Command;
use crate::strategy::strategies::flood_fill::FloodFillConfig;
use crate::strategy::strategy::StrategyEndState;
use crate::utils::map_display::{self, MapDisplay, MapDisplayWrite};
use crate::utils::nonempty::PotentiallyNonEmpty;
use crate::utils::path::PathReference;
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
        if goal.0 == world.mouse.pos {
            return StrategyComputationResult::Computed(Err(StrategyEndState::ReachedGoal));
        }

        let capsule_map = capsule::as_capsule_map(&world.map);

        let capsule_world = PartialWorldData::<N>::from(WorldData {
            map: capsule_map,
            mouse: world.mouse,
        });

        let flow_field = self.ff_strat.flow_field(&capsule_world);
        let mut shortest_known_path = match self.ff_strat.path_on_flow_field(&flow_field, *goal) {
            Ok(o) => o,
            Err(end) => return StrategyComputationResult::Computed(Err(end)),
        };

        #[cfg(feature = "internal_strat_logs")]
        {
            let mut map_display = MapDisplay::from(&capsule_world.map);
            let mut path_ref = PathReference::new(&shortest_known_path, &mut map_display);
            path_ref.set_char('*');
            info!(target: "strat/dkp", "Capsuled world: \n{map_display}");
        }

        let mut actions = vec![];

        while let Some(next_action) = shortest_known_path.one_towards_destination() {
            actions.push(next_action);
        }

        let actions = actions.into_iter().map(|a| ComputedAction {
            next_strategy_state: None,
            after_command: Command {
                ty: a,
                interrupts: vec![],
            },
        });

        let Some(actions) = actions.collect::<Vec<_>>().non_empty() else {
            return StrategyComputationResult::Computed(Err(StrategyEndState::NoPossibleAction(String::from("The goal was not reached, but the path has not next action to perform; Should not happen"))));
        };

        StrategyComputationResult::Computed(Ok(ComputedActions(actions)))
    }
}
