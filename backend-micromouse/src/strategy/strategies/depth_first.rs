use std::{
    collections::{HashMap, HashSet},
    usize,
};

use serde::{Deserialize, Serialize};
use tokio_util::either;
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
        strategies::utils::{
            depth_first_base::{
                DepthFirstBase, DepthFirstWithCurrent, MaybeInitDepthFirst, PathRanking,
            },
            value_map::ValueMap,
        },
        strategy::{
            ComputedAction, ComputedActions, FromConfig, GoalPosition, Strategy,
            StrategyComputationResult, StrategyEndState,
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
    pub path_ranking: PathRanking,
}

#[derive(Clone, Debug)]
pub struct DepthFirst<const N: usize> {
    df: MaybeInitDepthFirst,
    config: DepthFirstConfig,
}

impl<const N: usize> FromConfig<N> for DepthFirst<N> {
    type Config = DepthFirstConfig;

    #[instrument(
        name = "from_config DepthFirst",
        fields(description = "Create new DepthFirst-Strategy instance based on config")
    )]
    fn from_config(config: &Self::Config, starting_state: &WorldData<N>) -> Self {
        Self {
            df: MaybeInitDepthFirst::HasInitialStep(DepthFirstWithCurrent::new(starting_state)),
            config: config.clone(),
        }
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

        let df = match self.df {
            MaybeInitDepthFirst::HasInitialStep(ref with_initial) => {
                let (init_cmd, successor) = with_initial.clone().move_forward_from(
                    world,
                    world.mouse,
                    *goal,
                    Self::interrupt_right(self.config.path_ranking, world.mouse, *goal),
                    Self::interrupt_left(self.config.path_ranking, world.mouse, *goal),
                );
                return StrategyComputationResult::Computed(Ok(ComputedActions(NonEmpty::one(
                    ComputedAction {
                        next_strategy_state: Some(Self {
                            df: MaybeInitDepthFirst::WithoutCurrentStep(successor),
                            config: self.config.clone(),
                        }),
                        after_command: init_cmd,
                    },
                ))));
            }
            MaybeInitDepthFirst::WithoutCurrentStep(ref df) => df,
        };

        let Some(mut successor) = df.with_current_world(world, *goal) else {
            return StrategyComputationResult::NotEnoughInformation;
        };

        let (headless_moves, moves_dest) =
            match successor.moves_to_next_intersection(self.config.path_ranking, *goal) {
                Ok((moves, dest)) => (moves, dest),
                Err(strat_end) => return StrategyComputationResult::Computed(Err(strat_end)),
            };
        let (seeking_move, successor) = successor.move_forward_from(
            world,
            moves_dest,
            *goal,
            Self::interrupt_right(self.config.path_ranking, moves_dest, *goal),
            Self::interrupt_left(self.config.path_ranking, moves_dest, *goal),
        );

        let successor = Self {
            df: MaybeInitDepthFirst::WithoutCurrentStep(successor),
            config: self.config.clone(),
        };

        let actions: Vec<_> = headless_moves
            .into_iter()
            .map(|m| {
                let cmd = Command {
                    ty: m,
                    interrupts: vec![],
                };
                ComputedAction {
                    // Non-expandable / headless
                    next_strategy_state: None,
                    after_command: cmd,
                }
            })
            .chain(Some(ComputedAction {
                next_strategy_state: Some(successor),
                after_command: seeking_move,
            }))
            .collect();

        StrategyComputationResult::Computed(Ok(ComputedActions(
            actions.non_empty().expect("Command have to be non_empty"),
        )))
    }
}

impl<const N: usize> DepthFirst<N> {
    pub fn interrupt_left(
        path_ranking: PathRanking,
        current: MouseTransform,
        goal: GoalPosition,
    ) -> bool {
        if path_ranking != PathRanking::TowardsGoal {
            return false;
        }

        let vec_to_goal = goal.0 - current.pos;

        match current.dir {
            Direction::PosX => vec_to_goal.d_y < 0,
            Direction::PosY => vec_to_goal.d_x > 0,
            Direction::NegX => vec_to_goal.d_y > 0,
            Direction::NegY => vec_to_goal.d_x < 0,
        }
    }
    pub fn interrupt_right(
        path_ranking: PathRanking,
        current: MouseTransform,
        goal: GoalPosition,
    ) -> bool {
        if path_ranking != PathRanking::TowardsGoal {
            return false;
        }

        let vec_to_goal = goal.0 - current.pos;

        match current.dir {
            Direction::PosX => vec_to_goal.d_y > 0,
            Direction::PosY => vec_to_goal.d_x < 0,
            Direction::NegX => vec_to_goal.d_y < 0,
            Direction::NegY => vec_to_goal.d_x > 0,
        }
    }
}
