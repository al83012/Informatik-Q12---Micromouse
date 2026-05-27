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
        strategies::utils::value_map::ValueMap,
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

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DepthFirstConfig {
    pub forward_first: bool,
}

#[derive(Clone, Debug)]
pub struct DepthFirst<const N: usize> {
    intersection_stack: Vec<Position>,
    task_directions: HashMap<Position, ClumpedDFTask>,
    current_path: Path,
    config: DepthFirstConfig,
    // Current task only has to correspond to a move forward as all the rotations are handled with
    // the path backtracking which is added as a blind task (as the path traversal is
    // deterministic)
    current_task: DFTask,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct DFTask {
    try_direction: Direction,
    cell: Position,
    max_steps: usize,
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

    #[instrument(
        name = "from_config DepthFirst",
        fields(description = "Create new DepthFirst-Strategy instance based on config")
    )]
    fn from_config(config: &Self::Config, starting_state: &WorldData<N>) -> Self {
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
            current_task: DFTask {
                try_direction: starting_transf.dir,
                cell: starting_pos,
                max_steps: N,
            },
            intersection_stack: vec![starting_pos],
            task_directions,
            current_path: Path::new(starting_transf),
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
        let Some(intersections_of_current_step) = self.check_current_fully_measured(world) else {
            // INFO: NOT YET ENOUGH INFORMATION / THE MOUSE HAS NOT YET REACHED THE END OF THAT STEP OR
            // DOES NOT CERTAINLY KNOW IF THERE WILL BE MORE INTERSECTIONS
            return StrategyComputationResult::NotEnoughInformation;
        };

        let mut successor = self.clone();
        // The change made from the command leading to this result has not yet been applied
        successor.current_path.end_with(world.mouse);
        loop {
            let mut commands_emitted = vec![];

            // INFO: Termination state is fully known and measured

            // First add the new intersections
            successor.add_task_directions(&intersections_of_current_step);

            let next_intersection = {
                let Some(next_intersection) = successor.intersection_stack.last() else {
                    return StrategyComputationResult::Computed(Err(
                        StrategyEndState::NoPossibleAction(
                            "There is no more intersection to check".to_string(),
                        ),
                    ));
                };
                next_intersection.clone()
            };

            let potential_paths_next_intersection = successor
                .task_directions
                .get_mut(&next_intersection)
                .expect("Should be in here");

            let (do_moves, in_direction, new_path) = if successor.config.forward_first {
                // INFO: Planning the paths back to all the different directions of the next
                // intersection and then choosing the one with the lowest move-count
                potential_paths_next_intersection
                    .try_directions
                    .iter()
                    .map(|d| MouseTransform {
                        pos: next_intersection,
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
                let rand_dir = potential_paths_next_intersection
                    .try_directions
                    .iter()
                    .next()
                    .expect("Should have at least 1 element");
                let mut new_path = successor.current_path.clone();
                let moves = new_path
                    .return_to(MouseTransform {
                        pos: next_intersection,
                        dir: *rand_dir,
                    })
                    .expect("Should be accessible");

                (moves, *rand_dir, new_path)
            };

            // Mark the direction as handled
            potential_paths_next_intersection
                .try_directions
                .remove(&in_direction);

            if potential_paths_next_intersection.try_directions.is_empty() {
                // This intersection is finished after this step
                successor.task_directions.remove(&next_intersection);
                successor.intersection_stack.pop();
            }

            // TODO: Plan the move straight forward;

            let movement_ray = RayIterator::<N>::new(next_intersection, in_direction);

            let max_dist_till_wall = movement_ray.clone().count();

            let max_dist_till_visited = movement_ray
                .clone()
                .enumerate()
                .skip_while(|(_i, c)| {
                    *world.map.cell(&c.pos).expect("Should be there")
                        == CellDiscoveryStatus::Visited
                })
                .next()
                .map(|(i, _c)| i)
                .unwrap_or(usize::MAX);

            let max_dist_till_goal = movement_ray
                .enumerate()
                .skip_while(|(_i, c)| c.pos != goal.0)
                .next()
                .map(|(i, _c)| i)
                .unwrap_or(usize::MAX);

            let max_dist =
                usize::min(max_dist_till_goal, max_dist_till_visited).min(max_dist_till_wall);

            if max_dist == 0 {
                // The direction that was chosen was not constructive; check a new direction
                // (And delete the commands that would have been made to get into that position; it
                // is not important)
                continue;
            }

            // INFO: max_dist > 0 --> Actually do the move
            // First: apply move back along path (/rotate in spot)
            // WARN: We will not be able to adjust the path fully as we do not yet know where the
            // command will interrupt; When doing the next cmd-step, it will be given at the start
            let moves = successor
                .current_path
                .return_to(MouseTransform {
                    pos: next_intersection,
                    dir: in_direction,
                })
                .expect("Checked");

            commands_emitted.append(
                &mut moves
                    .into_iter()
                    .map(|m| Command {
                        ty: m,
                        interrupts: vec![],
                    })
                    .collect(),
            );

            // INFO: Then: Add the forward-searching-move to the end

            let interrupts = if self.config.forward_first {
                vec![
                    MeasurementInterrupt {
                        direction: RelativeDirection::Left,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::Continue,
                    },
                    MeasurementInterrupt {
                        direction: RelativeDirection::Forward,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfOpen,
                    },
                    MeasurementInterrupt {
                        direction: RelativeDirection::Right,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::Continue,
                    },
                ]
            } else {
                vec![
                    MeasurementInterrupt {
                        direction: RelativeDirection::Left,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfOpen,
                    },
                    MeasurementInterrupt {
                        direction: RelativeDirection::Forward,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfOpen,
                    },
                    MeasurementInterrupt {
                        direction: RelativeDirection::Right,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfOpen,
                    },
                ]
            };

            commands_emitted.push(Command {
                ty: MovementType::Move(max_dist as u8),
                interrupts,
            });

            let mut computed_actions: Vec<_> = commands_emitted
                .into_iter()
                .map(|command| ComputedAction {
                    next_strategy_state: None,
                    after_command: command,
                })
                .collect::<Vec<ComputedAction<N, Self>>>();

            // Make the last command in the chain have a head
            computed_actions
                .last_mut()
                .expect("At least 1 cmd")
                .next_strategy_state = Some(successor);

            return StrategyComputationResult::Computed(Ok(ComputedActions(
                computed_actions.non_empty().expect("At least 1 cmd"),
            )));
        }
    }
}

impl<const N: usize> DepthFirst<N> {
    /// Checks whether the task that was decided upon in the last step has been fully explored
    /// (This means, that all the walls left, right and fwd are known on the path (or at least the
    /// path up to a premature termination)
    ///
    /// Returns None, if the measurements that are present for the current task are not enough for
    /// getting all the possible intersections on the way
    ///
    /// Returns Some and a list of possible intersections on the way (without removing those that
    /// have already been done)
    ///
    ///
    /// To conclude: The function checks not only if the
    #[instrument(
        name = "check_current_fully_measured",
        fields(
            description = "Check whether the given command is fully measured, i.e. if all the walls along its path are known"
        )
    )]
    pub fn check_current_fully_measured(
        &self,
        projected_termination_state: &PartialWorldData<N>,
    ) -> Option<Vec<MouseTransform>> {
        let current_task = self.current_task.clone();
        let from_pos = current_task.cell;
        let in_dir = current_task.try_direction;
        let max_steps = current_task.max_steps;

        let mut intersections = Vec::new();

        let mut transf_move = TransformedMovement::new(
            MovementType::Move(max_steps as u8),
            MouseTransform {
                pos: from_pos,
                dir: in_dir,
            },
        );

        let map = projected_termination_state.map;

        for i in 0..=max_steps {
            let transf_at_step = transf_move.at_step(i).expect("Inside max_step");
            let pos_at_step = transf_at_step.pos;

            let dir_right = RelativeDirection::Right.transform_by(&in_dir);
            let dir_fwd = RelativeDirection::Forward.transform_by(&in_dir);
            let dir_left = RelativeDirection::Left.transform_by(&in_dir);

            let right = map
                .wall(&pos_at_step, &dir_right)
                .unwrap_or(&WallDiscoveryStatus::Exists(true));

            let fwd = map
                .wall(&pos_at_step, &dir_fwd)
                .unwrap_or(&WallDiscoveryStatus::Exists(true));
            let left = map
                .wall(&pos_at_step, &dir_left)
                .unwrap_or(&WallDiscoveryStatus::Exists(true));

            let not_fully_discovered = *right == WallDiscoveryStatus::Undiscovered
                || *fwd == WallDiscoveryStatus::Undiscovered
                || *left == WallDiscoveryStatus::Undiscovered;

            if not_fully_discovered {
                return None;
            }

            match right {
                WallDiscoveryStatus::Visited | WallDiscoveryStatus::Exists(false) => {
                    if Self::is_visitable(map, pos_at_step, dir_right) {
                        intersections.push(MouseTransform {
                            pos: pos_at_step,
                            dir: dir_right,
                        });
                    }
                }
                _ => {}
            }
            match fwd {
                WallDiscoveryStatus::Visited | WallDiscoveryStatus::Exists(false) => {
                    if Self::is_visitable(map, pos_at_step, dir_fwd) {
                        intersections.push(MouseTransform {
                            pos: pos_at_step,
                            dir: dir_fwd,
                        });
                    }
                }
                _ => {}
            }
            match left {
                WallDiscoveryStatus::Visited | WallDiscoveryStatus::Exists(false) => {
                    if Self::is_visitable(map, pos_at_step, dir_left) {
                        intersections.push(MouseTransform {
                            pos: pos_at_step,
                            dir: dir_left,
                        });
                    }
                }
                _ => {}
            }

            if transf_at_step == projected_termination_state.mouse {
                // premature exit `projected_termination_state.mouse` = step of termination
                break;
            }
        }

        // todo!("I acutally still need to check that there is not a visited cell behind those walls");

        Some(intersections)
    }

    #[instrument(
        name = "add_task_directions",
        skip(branches),
        fields(description = "Add the given directions to the exploration-stack")
    )]
    pub fn add_task_directions<'a>(
        &mut self,
        branches: impl IntoIterator<Item = &'a MouseTransform>,
    ) {
        for branch in branches.into_iter() {
            if let Some(intersection_tasks) = self.task_directions.get_mut(&branch.pos) {
                intersection_tasks.try_directions.insert(branch.dir);
            } else {
                self.task_directions.insert(
                    branch.pos,
                    ClumpedDFTask {
                        try_directions: HashSet::from_iter(vec![branch.dir]),
                    },
                );
                self.intersection_stack.push(branch.pos);
            }
        }
    }

    #[instrument(name = "is_visitable", fields(description = "Check whether a given cell is visitable (non visited + inside map)"))]
    pub fn is_visitable(map: Map<N>, from_cell: Position, direction: Direction) -> bool {
        let d_offset = DirectionNormalizedVector::from(direction);

        let x: i32 = from_cell.x as i32 + d_offset.x as i32;
        let y: i32 = from_cell.y as i32 + d_offset.y as i32;

        if x < 0 || y < 0 {
            return false;
        }

        let Some(cell) = map.cell(&Position {
            x: x as u32,
            y: y as u32,
        }) else {
            return false;
        };

        *cell != CellDiscoveryStatus::Visited
    }
}
