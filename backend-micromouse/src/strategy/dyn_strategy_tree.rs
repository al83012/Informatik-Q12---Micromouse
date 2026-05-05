use crate::strategy::{
    strategies::{
        breadth_first::BreadthFirst, depth_first::DepthFirst, flood_fill::FloodFill,
        follow_wall::FollowWall, random_move::RandomMove,
    },
    strategy::{FromConfig, GoalPosition, Strategy},
    strategy_tree::{
        StrategyStart, StrategyTree, StrategyTreeConfig, TreeCreationError, TreeCreationSuccess,
    },
};

pub enum DynStrategyTree<const N: usize> {
    DepthFirst(StrategyTree<N, DepthFirst<N>>),
    BreadthFirst(StrategyTree<N, BreadthFirst<N>>),
    FollowWall(StrategyTree<N, FollowWall<N>>),
    FloodFill(StrategyTree<N, FloodFill<N>>),
    RandomMove(StrategyTree<N, RandomMove<N>>),
}

pub enum DynStrategyTreeConfig<const N: usize> {
    DepthFirst(<DepthFirst<N> as FromConfig<N>>::Config),
    BreadthFirst(<BreadthFirst<N> as FromConfig<N>>::Config),
    FollowWall(<FollowWall<N> as FromConfig<N>>::Config),
    FloodFill(<FloodFill<N> as FromConfig<N>>::Config),
    RandomMove(<RandomMove<N> as FromConfig<N>>::Config),
}


impl<const N: usize> DynStrategyTree<N> {
    pub fn new(
        starting_condition: StrategyStart<N>,
        tree_config: DynStrategyTreeConfig<N>,
        goal_position: GoalPosition) -> Self {
        todo!()
    }
}
