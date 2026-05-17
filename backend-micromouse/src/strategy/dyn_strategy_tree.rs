use std::sync::mpsc;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{
    comm::micromouse_message::Command,
    map::world_data::WorldData,
    strategy::{
        strategies::{
            breadth_first::BreadthFirst, dbg_known_path::DbgKnownPath, depth_first::DepthFirst,
            flood_fill::FloodFill, follow_wall::FollowWall, random_move::RandomMove,
        },
        strategy::{FromConfig, GoalPosition, Strategy},
        strategy_tree::{
            SentUnfinishedCommands, StrategyStart, StrategyTree, StrategyTreeConfig,
            StrategyTreeError, TreeCreationError, TreeCreationSuccess,
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

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum DynStrategyConfig<const N: usize> {
    DepthFirst(<DepthFirst<N> as FromConfig<N>>::Config),
    BreadthFirst(<BreadthFirst<N> as FromConfig<N>>::Config),
    FollowWall(<FollowWall<N> as FromConfig<N>>::Config),
    FloodFill(<FloodFill<N> as FromConfig<N>>::Config),
    RandomMove(<RandomMove<N> as FromConfig<N>>::Config),
    DbgKnownPath(<DbgKnownPath<N> as FromConfig<N>>::Config),
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct StrategyChangeCommand<const N: usize> {
    pub set_postion: Option<MouseTransform>,
    pub reset_map: bool,
    pub set_strategy: Option<DynStrategyConfig<N>>,
    pub set_goal: Option<GoalPosition>,
}

impl<const N: usize> DynStrategyTreeManager<N> {
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

    /// Is unsafe as it leaves self in the improper "Closed"-State
    unsafe fn erase_strat(&mut self) -> Option<SentUnfinishedCommands<N>> {
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

    fn send_cmd(&mut self, cmd: Command) {
        self.command_sender
            .send(cmd)
            .expect("Channel should not be closed");
    }

    async fn await_cmd(&mut self) -> Command {
        self.command_receiver
            .recv()
            .await
            .expect("Channel should not be closed")
    }

    // pub fn apply_measurement(&mut self)

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

        if reset_map {
            self.current_world.mouse = current_mouse;
            self.current_world = self.current_world.only_pos();
            // Still need to take over the sent cmds and the end world of those is the new pos
        } else {
            // self.current_world = self.current_world;
        }
        let strategy_start = if let Some(erased) = erased {
            StrategyStart::ContinueAfterDoing {
                after_cmds: erased,
                reset_world: reset_map,
            }
        } else {
            StrategyStart::DirectlyAtState(self.current_world.clone())
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
