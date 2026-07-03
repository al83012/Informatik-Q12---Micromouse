use std::sync::mpsc;

use serde::{Deserialize, Serialize};
use tokio::sync::{
    broadcast::{Receiver, Sender},
    mpsc::{UnboundedReceiver, UnboundedSender},
};
use tracing::{info, instrument, warn};

use crate::{
    comm::micromouse_message::{Command, NonIndexMicromouseMessage},
    map::{
        command_world_state::RejectedOutcomes, map::Map, measurement::Measurement,
        world_data::WorldData,
    },
    strategy::{
        strategies::{
            breadth_first::BreadthFirst, dbg_known_path::DbgKnownPath, depth_first::DepthFirst,
            flood_fill::FloodFill, follow_wall::FollowWall, random_move::RandomMove,
        },
        strategy::{FromConfig, GoalPosition, Strategy, StrategyEndState},
        strategy_tree::{
            self, FinishRootError, GraftingFilter, PruneError, SentUnfinishedCommands,
            StrategyStart, StrategyTree, StrategyTreeConfig, StrategyTreeError, TreeCreationError,
            TreeCreationInitialEffect, TreeCreationSuccess,
        },
        visuals::FrontendVisuals,
    },
    transform::position::{MouseTransform, Position},
};

pub enum DynStrategyTree<const N: usize> {
    DepthFirst(StrategyTree<N, DepthFirst<N>>),
    BreadthFirst(StrategyTree<N, BreadthFirst<N>>),
    FollowWall(StrategyTree<N, FollowWall<N>>),
    FloodFill(StrategyTree<N, FloodFill<N>>),
    RandomMove(StrategyTree<N, RandomMove<N>>),
    DbgKnownPath(StrategyTree<N, DbgKnownPath<N>>),
    Closed(FrontendVisuals),
    CurrentlyChanging,
}

pub struct DynStrategyTreeManager<const N: usize> {
    strategy_tree: DynStrategyTree<N>,
    command_sender: UnboundedSender<NonIndexMicromouseMessage>,
    command_receiver: UnboundedReceiver<NonIndexMicromouseMessage>,

    // We are separately storing the current world in order to be able to plug it into the
    // StartingState if necessary; The strategy tree itself only stores the world where interrupts
    // could interact with execution
    current_world: WorldData<N>,

    goal_pos: GoalPosition,
    strat_config: DynStrategyConfig<N>,

    desired_depth: usize,
    max_nodes: usize,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub enum DynStrategyConfig<const N: usize> {
    DepthFirst(<DepthFirst<N> as FromConfig<N>>::Config),
    BreadthFirst(<BreadthFirst<N> as FromConfig<N>>::Config),
    FollowWall(<FollowWall<N> as FromConfig<N>>::Config),
    FloodFill(<FloodFill<N> as FromConfig<N>>::Config),
    RandomMove(<RandomMove<N> as FromConfig<N>>::Config),
    DbgKnownPath(<DbgKnownPath<N> as FromConfig<N>>::Config),
    Closed,
}

impl<const N: usize> DynStrategyConfig<N> {
    pub fn require_grafting_filter(&self) -> GraftingFilter {
        macro_rules! require_grafting_filter {
            ([$($variant:ident),+]) => {
                {
                use super::strategy::WithGraftingFilter;
                match self {
                        $(DynStrategyConfig::<N>::$variant(val) => {
                        WithGraftingFilter::require_grafting_filter(val)
                    })+
                    DynStrategyConfig::Closed => {
                        GraftingFilter::None
                    }
                }

                }
                }
            }

        require_grafting_filter!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ])
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct StrategyChangeCommand<const N: usize> {
    pub reset_map: bool,
    pub set_strategy: Option<DynStrategyConfig<N>>,
    pub set_goal: Option<GoalPosition>,
}

impl<const N: usize> DynStrategyTreeManager<N> {
    #[instrument(
        skip(visuals),
        name = "new DynStrategyTreeManager",
        fields(description = "Create new Strategy Tree Manager")
    )]
    pub fn new(
        starting_condition: WorldData<N>,
        goal_position: GoalPosition,
        desired_depth: usize,
        max_nodes: usize,
        visuals: FrontendVisuals,
    ) -> Self {
        // WARN: Will only become active once .modify is called the first time
        let strategy_tree = DynStrategyTree::Closed(visuals);

        let (command_sender, command_receiver) = tokio::sync::mpsc::unbounded_channel();
        Self {
            strategy_tree,
            command_sender,
            command_receiver,
            current_world: starting_condition,
            goal_pos: goal_position,
            strat_config: DynStrategyConfig::Closed,
            desired_depth,
            max_nodes,
        }
    }

    ///  WARN: Leaves self.strategy_tree in the Closed-Variant
    #[instrument(
        skip(self),
        name = "erase_strat",
        fields(description = "Erase the last strategy, enabling the application of a new one")
    )]
    unsafe fn erase_strat(&mut self) -> (FrontendVisuals, SentUnfinishedCommands<N>) {
        info!(target: "strat", "ERASING STRATEGY");
        macro_rules! erase_strat {
            ([$($variant:ident),+]) => {
                {
                    let mut current_val = DynStrategyTree::<N>::CurrentlyChanging;
                    std::mem::swap(&mut self.strategy_tree, &mut current_val);
                    match current_val {
                        $(
                            DynStrategyTree::<N>::$variant(val) => {
                                val.close()
                            }
                        )*
                        DynStrategyTree::<N>::Closed(visuals) => (visuals, SentUnfinishedCommands::HasBlockingRoot{world: self.current_world.clone()}),
                        DynStrategyTree::<N>::CurrentlyChanging => panic!("Erasing an already changing strategy tree; NOT ALLOWED"),
                    }
                }
            };
        }

        let e = erase_strat!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ]);
        info!(target: "strat", "FINISHED ERASING");
        e
    }

    #[instrument(
        skip(self, visuals),
        name = "set_starting_cond",
        fields(description = "Set new strategy start (given an already running strat manager)")
    )]
    fn set_starting_cond(
        &mut self,
        starting_condition: StrategyStart<N>,
        strategy_config: DynStrategyConfig<N>,
        goal_position: GoalPosition,
        desired_depth: usize,
        max_nodes: usize,
        visuals: FrontendVisuals,
    ) -> Result<Option<StrategyEndState>, TreeCreationError> {
        info!(target: "strat", "SET STARTING CONDITION for {strategy_config:?}");
        macro_rules! new_tree {
            ([$($variant:ident),+]) => {
                match strategy_config {
                    $(DynStrategyConfig::$variant(val) => {
                        let strat_conf = StrategyTreeConfig{
                            strategy_config: val,
                            desired_depth,
                            max_nodes
                        };
                        let tree = StrategyTree::new(starting_condition, strat_conf, goal_position, visuals)?;
                        let TreeCreationSuccess{tree, origin_command_or_end} = tree;
                        (DynStrategyTree::<N>::$variant(tree), origin_command_or_end)
                    })+
                    DynStrategyConfig::Closed => {
                        (DynStrategyTree::Closed(visuals), TreeCreationInitialEffect::OriginCommand(None))
                    }
                }
            };
        }

        let (new_tree, cmd_or_end) = new_tree!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ]);

        let variant = std::mem::discriminant(&new_tree);
        info!(target: "strat", "NEW DYN TREE: {variant:?}");
        self.strategy_tree = new_tree;

        match cmd_or_end {
            TreeCreationInitialEffect::ImmediateEnd(strategy_end) => {
                return Ok(Some(strategy_end));
            }
            TreeCreationInitialEffect::OriginCommand(Some(cmd)) => {
                self.send_cmd(cmd.clone());
            }
            _ => {}
        }

        Ok(None)
    }

    #[instrument(
        skip(self),
        name = "send_cmd",
        fields(description = "Add cmd to internal queue")
    )]
    fn send_cmd(&mut self, cmd: Command) {
        self.command_sender
            .send(NonIndexMicromouseMessage::Command(cmd.clone()))
            .expect("Channel should not be closed");
        info!(target: "strat", "Queued dyn strategy cmd {cmd:?}");
    }
    fn send_restart_confirm(&mut self) {
        self.command_sender
            .send(NonIndexMicromouseMessage::RestartConfirm)
            .expect("Channel should not be closed");
        info!(target: "strat", "Sent Restart-Confirm");
    }
    fn send_reset_map(&mut self) {
        self.command_sender
            .send(NonIndexMicromouseMessage::ResetMapAndPos)
            .expect("Channel should not be closed");
        info!(target: "strat", "Sent Restart-Confirm");
    }

    #[instrument(
        skip(self),
        name = "await_cmd",
        fields(description = "Wait for new command from the queue")
    )]
    pub async fn await_cmd(&mut self) -> NonIndexMicromouseMessage {
        self.command_receiver
            .recv()
            .await
            .expect("Channel should not be closed")
    }

    /// Called upon an update_map-Event being sent
    #[instrument(
        skip(self),
        name = "update_filter",
        fields(
            description = "Do a full filter-update (update strategy tree using map-information)"
        )
    )]
    pub fn update_filter(
        &mut self,
        map: &Map<N>,
    ) -> Result</*Vec<Command>*/ (), StrategyTreeError> {
        info!(target: "strat", "UPDATE FILTER");
        macro_rules! update_filter {
            ([$($variant:ident),+]) => {

                {
                let prune_result = match self.strategy_tree {
                        $(DynStrategyTree::$variant(ref mut tree) => {
                            tree.handle_map_update(map)
                            // tree.prune_not_potentially_eq(&partial)
                        },)+
                        DynStrategyTree::Closed(ref _visuals) => {
                            warn!(target: "strat", "Updating closed tree");
                            return Ok(());
                        },
                        DynStrategyTree::CurrentlyChanging => panic!("Strategy Tree is currently changing; NOT ALLOWED"),
                    };
                prune_result
                }

            }
        }

        let res = update_filter!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ])?;

        for c in res {
            self.send_cmd(c);
        }

        Ok(())
    }

    #[instrument(
        skip(self),
        name = "prune_current",
        fields(description = "Prune using the given RejectedOutcomes")
    )]
    pub fn prune_current(
        &mut self,
        rejected: &RejectedOutcomes,
    ) -> Result</*Vec<Command>*/ (), StrategyTreeError> {
        info!(target: "strat", "PRUNE REJECTED {rejected:?}");
        macro_rules! prune_current {
            ([$($variant:ident),+]) => {

                {
                let prune_result = match self.strategy_tree {
                        $(DynStrategyTree::$variant(ref mut tree) => {
                            tree.handle_cmd_rejection(rejected)
                        },)+
                        DynStrategyTree::Closed(ref _visuals) => {
                            info!(target: "strat", "Pruning closed tree");
                            return Ok(())
                        }
                        DynStrategyTree::CurrentlyChanging => panic!("Pruning tree which is currently changing; NOT ALLOWED")
                ,
                    };
                prune_result
                }

            }
        }

        let res = prune_current!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ])?;

        for c in res {
            self.send_cmd(c);
        }

        Ok(())
    }

    #[instrument(
        skip(self),
        name = "set_pos_to_start_and_restart",
        fields(
            description = "Clear the entire strategy state and make it assume the default starting position (Does not reset command queue of the micromouse); Restarts current strategy"
        )
    )]
    pub fn set_pos_to_origin_and_restart(&mut self) -> Result<(), StrategyTreeError> {
        info!(target: "strat", "RESTART STRATEGY MANAGER");
        let (visuals, _erased) = unsafe { self.erase_strat() };
        let strat_config = self.strat_config.clone();
        let goal_pos = self.goal_pos;
        let default_world = WorldData::default().only_pos();
        *self = Self::new(
            default_world,
            self.goal_pos,
            self.desired_depth,
            self.max_nodes,
            visuals,
        );
        // Putting in the restart-marker before the new commands --> Making the non-sent commands
        // inside the queue invalid
        self.send_restart_confirm();
        self.send_reset_map();
        self.modify(StrategyChangeCommand {
            reset_map: false,
            set_strategy: Some(strat_config),
            set_goal: Some(goal_pos),
        })?;
        Ok(())
    }

    #[instrument(
        skip(self),
        name = "finish_current_cmd",
        fields(description = "React to command completion (assume root to be finished)")
    )]
    pub fn finish_current_cmd(&mut self) -> Result<Option<StrategyEndState>, StrategyTreeError> {
        info!(target: "strat", "CHANGE CURRENT COMMAND");
        macro_rules! finish_current_cmd {
            ([$($variant:ident),+]) => {

                {
                let finish_root_result = match self.strategy_tree {
                        $(DynStrategyTree::$variant(ref mut tree) => {
                            tree.handle_finish_root()
                        },)+
                        DynStrategyTree::Closed(ref _visuals) => panic!("Closed has no root to be finished"),
                        DynStrategyTree::CurrentlyChanging => panic!("Finishing Root on tree that is currently changing")
                    };
                finish_root_result
                }

            }
        }
        match finish_current_cmd!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ]) {
            Ok(v) => {
                for c in v {
                    self.send_cmd(c);
                }

                Ok(None)
            }
            Err(e) => match e {
                StrategyTreeError::WhileFinishingCommand(FinishRootError::SuccessorIsEnd(
                    end_state,
                )) => Ok(Some(end_state)),
                _ => Err(e),
            },
        }
    }

    // pub fn apply_measurement(&mut self)

    #[instrument(
        skip(self),
        name = "modify",
        fields(description = "Freely change the current strategy (erasing old one)")
    )]
    pub fn modify(
        &mut self,
        change: StrategyChangeCommand<N>,
    ) -> Result<Option<StrategyEndState>, TreeCreationError> {
        info!(target: "strat", "MODIFY {change:?}");
        let StrategyChangeCommand {
            reset_map,
            set_strategy,
            set_goal,
        } = change;

        self.strat_config = set_strategy.clone().unwrap_or(self.strat_config.clone());

        // let current_mouse = set_postion.unwrap_or(self.current_world.mouse);

        self.goal_pos = set_goal.unwrap_or(self.goal_pos);

        let (visuals, erased) = unsafe { self.erase_strat() };

        let strategy_start = StrategyStart::ContinueAfterDoing {
            after_cmds: erased,
            grafting_filter: if reset_map {
                GraftingFilter::RemoveAll
            } else {
                self.strat_config.require_grafting_filter()
            },
        };

        if reset_map {
            self.send_reset_map();
        }

        let res = self.set_starting_cond(
            strategy_start,
            self.strat_config.clone(),
            self.goal_pos,
            self.desired_depth,
            self.max_nodes,
            visuals,
        );
        info!(target: "strat", "FINISHED MODIFY: {:?}", std::mem::discriminant(&self.strategy_tree));
        res
    }
}
