use std::collections::{HashMap, HashSet};

use crate::{
    map::world_data::WorldData,
    strategy::{
        strategies::utils::value_map::ValueMap,
        strategy::{FromConfig, Strategy, StrategyComputationResult, StrategyEndState},
    },
    transform::{
        direction::Direction,
        position::{MouseTransform, Position},
    },
    utils::{nonempty::NonEmpty, path::Path},
};

#[derive(Debug, Clone)]
pub struct DepthFirstConfig {
    pub forward_first: bool,
}

#[derive(Clone)]
pub struct DepthFirst<const N: usize> {
    intersection_stack: Vec<Position>,
    task_directions: HashMap<Position, ClumpedDFTask>,
    current_path: Path,
    config: DepthFirstConfig,
    visited_marker: ValueMap<N, bool>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct DFTask {
    try_direction: Direction,
    cell: Position,
}

/// Like DF-Task, but clumped together to represent our ability to decide, which intersection to
/// process first (e.g. greedily taking the straight path first)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ClumpedDFTask {
    try_directions: HashSet<Direction>,
}

impl ClumpedDFTask {
    pub fn add_direction(&mut self, direction: Direction) {
        self.try_directions.insert(direction);
    }
}

impl<const N: usize> FromConfig<N> for DepthFirst<N> {
    type Config = DepthFirstConfig;

    fn from_config(config: Self::Config, starting_state: WorldData<N>) -> Self {
        let starting_transf = starting_state.mouse;
        let starting_pos = starting_transf.pos;
        let mut task_directions = HashMap::new();
        task_directions.insert(
            starting_pos,
            ClumpedDFTask {
                try_directions: HashSet::from_iter(vec![
                    Direction::PosX,
                    Direction::PosY,
                    Direction::NegX,
                    Direction::NegY,
                ]),
            },
        );
        let mut visited_marker = ValueMap::<N, bool>::new(false);
        *visited_marker
            .value_mut(starting_pos)
            .expect("starting pos in bounds") = true;
        Self {
            intersection_stack: vec![starting_pos],
            task_directions,
            current_path: Path::new(starting_transf),
            config,
            visited_marker,
        }
    }
}

impl<const N: usize> DepthFirst<N> {
    pub fn add_task_to_queue(&mut self, task: DFTask) {
        todo!()
    }
}

impl<const N: usize> Strategy<N> for DepthFirst<N> {
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,

        // The goal is only important for determining, whether the mouse has reached its goal
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        let mut successor = self.clone();

        let Some(next_intersection) = successor.intersection_stack.last() else {
            return StrategyComputationResult::Computed(Err(StrategyEndState::NoPossibleAction(
                "There is no more intersection to check".to_string(),
            )));
        };

        let potential_paths = successor
            .task_directions
            .get_mut(next_intersection)
            .expect("Should be in here");

        let (do_moves, in_direction, new_path) = if successor.config.forward_first {
            potential_paths
                .try_directions
                .iter()
                .map(|d| MouseTransform {
                    pos: *next_intersection,
                    dir: *d,
                })
                .map(|t| {
                    let mut new_path = successor.current_path.clone();
                    (
                        new_path
                            .return_to(t)
                            .expect("Should be accessible (Since the path was visited)"),
                        t.dir,
                        new_path,
                    )
                })
                .min_by_key(|(v, _, _)| v.len())
                .expect("Should have at least 1 element")
        } else {
            let rand_dir = potential_paths
                .try_directions
                .iter()
                .next()
                .expect("Should have at least 1 element");
            let mut new_path = self.current_path.clone();
            let moves = new_path
                .return_to(MouseTransform {
                    pos: *next_intersection,
                    dir: *rand_dir,
                })
                .expect("Should be accessible");

            (moves, *rand_dir, new_path)
        };

        potential_paths.try_directions.remove(&in_direction);

        if potential_paths.try_directions.is_empty() {
            // This intersection is finished after this step
            successor.task_directions.remove(next_intersection);
            successor.intersection_stack.pop();
        }

        

        todo!()
    }
}
