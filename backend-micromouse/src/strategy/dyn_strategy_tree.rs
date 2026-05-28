use std::sync::mpsc;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::instrument;

use crate::{
    comm::micromouse_message::Command,
    map::{map::Map, measurement::Measurement, world_data::WorldData},
    strategy::{
        strategies::{
            breadth_first::BreadthFirst, dbg_known_path::DbgKnownPath, depth_first::DepthFirst,
            flood_fill::FloodFill, follow_wall::FollowWall, random_move::RandomMove,
        },
        strategy::{FromConfig, GoalPosition, Strategy, StrategyEndState},
        strategy_tree::{
            FinishRootError, PruneError, SentUnfinishedCommands, StrategyStart, StrategyTree,
            StrategyTreeConfig, StrategyTreeError, TreeCreationError, TreeCreationSuccess,
        },
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
    /// Value used to move out of self temporarily; Immediately panic if it ever comes up in
    /// "common" use
    Closed,
}

pub struct DynStrategyTreeManager<const N: usize> {
    strategy_tree: DynStrategyTree<N>,
    command_sender: Sender<Command>,
    command_receiver: Receiver<Command>,

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
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct StrategyChangeCommand<const N: usize> {
    pub set_postion: Option<MouseTransform>,
    pub reset_map: bool,
    pub set_strategy: Option<DynStrategyConfig<N>>,
    pub set_goal: Option<GoalPosition>,
}

impl<const N: usize> DynStrategyTreeManager<N> {
    #[instrument(
        name = "new DynStrategyTreeManager",
        fields(description = "Create new Strategy Tree Manager")
    )]
    pub fn new(
        starting_condition: WorldData<N>,
        strategy_config: DynStrategyConfig<N>,
        goal_position: GoalPosition,
        desired_depth: usize,
        max_nodes: usize,
    ) -> Result<Self, StrategyTreeError> {
        Self::new_starting_cond(
            starting_condition,
            strategy_config,
            goal_position,
            desired_depth,
            max_nodes,
        )
    }

    ///  WARN: Leaves self.strategy_tree in the Closed-Variant
    #[instrument(
        skip(self),
        name = "erase_strat",
        fields(description = "Erase the last strategy, enabling the application of a new one")
    )]
    unsafe fn erase_strat(&mut self) -> SentUnfinishedCommands<N> {
        macro_rules! erase_strat {
            ([$($variant:ident),+]) => {
                {
                    let mut current_val = DynStrategyTree::<N>::Closed;
                    std::mem::swap(&mut self.strategy_tree, &mut current_val);
                    match current_val {
                        $(
                            DynStrategyTree::<N>::$variant(val) => {
                                val.close()
                            }
                        )*
                        DynStrategyTree::<N>::Closed => panic!("Closed is not a proper state; It should only appear in operations and not be constructable")
                    }
                }
            };
        }

        erase_strat!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ])
    }

    #[instrument(
        skip(self),
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
    ) -> Result<(), StrategyTreeError> {
        macro_rules! new_tree {
            ([$($variant:ident),+]) => {
                match strategy_config {
                    $(DynStrategyConfig::$variant(val) => {
                        let strat_conf = StrategyTreeConfig{
                            strategy_config: val,
                            desired_depth,
                            max_nodes
                        };
                        let tree = StrategyTree::new(starting_condition, strat_conf, goal_position)?;
                        let TreeCreationSuccess{tree, origin_command} = tree;
                        (DynStrategyTree::<N>::$variant(tree), origin_command)
                    })+
                }
            };
        }

        let (new_tree, cmd) = new_tree!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ]);

        self.strategy_tree = new_tree;

        if let Some(cmd) = cmd {
            self.send_cmd(cmd);
        }

        Ok(())
    }

    #[instrument(
        name = "new_starting_cond",
        fields(description = "Create entirely new Strategy Manager with given conditions")
    )]
    fn new_starting_cond(
        starting_condition: WorldData<N>,
        strategy_config: DynStrategyConfig<N>,
        goal_position: GoalPosition,
        desired_depth: usize,
        max_nodes: usize,
    ) -> Result<Self, StrategyTreeError> {
        macro_rules! new_tree {
            ([$($variant:ident),+]) => {
                match strategy_config.clone() {
                    $(DynStrategyConfig::$variant(val) => {
                        let strat_conf = StrategyTreeConfig{
                            strategy_config: val,
                            desired_depth,
                            max_nodes
                        };
                        let tree = StrategyTree::new(StrategyStart::DirectlyAtState(starting_condition.clone()), strat_conf, goal_position)?;
                        let TreeCreationSuccess{tree, origin_command} = tree;
                        (DynStrategyTree::<N>::$variant(tree), origin_command)
                    })+
                }
            };
        }

        let (new_tree, cmd) = new_tree!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ]);

        let (command_sender, command_receiver) = tokio::sync::broadcast::channel(16);

        let mut res = Self {
            strategy_tree: new_tree,
            command_sender,
            command_receiver,
            current_world: starting_condition,
            goal_pos: goal_position,
            strat_config: strategy_config,
            desired_depth,
            max_nodes,
        };

        if let Some(cmd) = cmd {
            res.send_cmd(cmd);
        }

        Ok(res)
    }

    #[instrument(
        skip(self),
        name = "send_cmd",
        fields(description = "Add cmd to internal queue")
    )]
    fn send_cmd(&mut self, cmd: Command) {
        self.command_sender
            .send(cmd)
            .expect("Channel should not be closed");
    }

    #[instrument(
        skip(self),
        name = "await_cmd",
        fields(description = "Wait for new command from the queue")
    )]
    pub async fn await_cmd(&mut self) -> Command {
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
    pub fn update_filter(&mut self, map: Map<N>) -> Result<Vec<Command>, StrategyTreeError> {
        let partial = map.into();
        macro_rules! update_filter {
            ([$($variant:ident),+]) => {

                {
                let prune_result = match self.strategy_tree {
                        $(DynStrategyTree::$variant(ref mut tree) => {
                            tree.handle_map_update(&partial)
                            // tree.prune_not_potentially_eq(&partial)
                        },)+
                        DynStrategyTree::Closed => panic!("Closed is not a proper state; It should only appear in operations and not be constructable"),
                    };
                prune_result
                }

            }
        }

        update_filter!([
            DepthFirst,
            BreadthFirst,
            FollowWall,
            FloodFill,
            RandomMove,
            DbgKnownPath
        ])
    }

    #[instrument(
        skip(self),
        name = "set_pos_to_start_and_restart",
        fields(
            description = "Clear the entire strategy state and make it assume the default starting position (Does not reset command queue of the micromouse); Restarts current strategy"
        )
    )]
    pub fn set_pos_to_start_and_restart(&mut self) -> Result<(), StrategyTreeError> {
        self.modify(StrategyChangeCommand {
            set_postion: Some(MouseTransform {
                pos: Position { x: 0, y: 0 },
                dir: crate::transform::direction::Direction::PosX,
            }),
            reset_map: true,
            set_strategy: Some(self.strat_config.clone()),
            set_goal: Some(self.goal_pos),
        })
    }

    #[instrument(
        skip(self),
        name = "update_pos",
        fields(description = "Overwrite the postion that is assumed; Restarts current strategy")
    )]
    pub fn update_pos(&mut self, transform: MouseTransform) -> Result<(), StrategyTreeError> 
    {
        self.modify(StrategyChangeCommand {
            set_postion: Some(transform),
            reset_map: false,
            set_strategy: Some(self.strat_config.clone()),
            set_goal: Some(self.goal_pos),
        })
    }

    #[instrument(
        skip(self),
        name = "finish_current_cmd",
        fields(description = "React to command completion (assume root to be finished)")
    )]
    pub fn finish_current_cmd(&mut self) -> Result<Option<StrategyEndState>, FinishRootError> 
    {
        macro_rules! finish_current_cmd {
            ([$($variant:ident),+]) => {

                {
                let finish_root_result = match self.strategy_tree {
                        $(DynStrategyTree::$variant(ref mut tree) => {
                            tree.finish_root()
                        },)+
                        DynStrategyTree::Closed => panic!("Closed is not a proper state; It should only appear in operations and not be constructable"),
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
            Ok(()) => Ok(None),
            Err(e) => match e {
                FinishRootError::SuccessorIsEnd(end_state) => Ok(Some(end_state)),
                _ => Err(e),
            },
        }
    }

    // pub fn apply_measurement(&mut self)

    #[instrument(skip(self), name = "modify", fields(description = "Freely change the current strategy (erasing old one)"))]
    pub fn modify(&mut self, change: StrategyChangeCommand<N>) -> Result<(), StrategyTreeError> {
        let StrategyChangeCommand {
            set_postion,
            reset_map,
            set_strategy,
            set_goal,
        } = change;

        let current_mouse = set_postion.unwrap_or(self.current_world.mouse);

        self.goal_pos = set_goal.unwrap_or(self.goal_pos);

        let erased = unsafe { self.erase_strat() };


        let strategy_start = StrategyStart::ContinueAfterDoing {
            after_cmds: erased,
            reset_world: reset_map,
        };

        self.strat_config = set_strategy.clone().unwrap_or(self.strat_config.clone());

        self.set_starting_cond(
            strategy_start,
            self.strat_config.clone(),
            self.goal_pos,
            self.desired_depth,
            self.max_nodes,
        )
    }
}
