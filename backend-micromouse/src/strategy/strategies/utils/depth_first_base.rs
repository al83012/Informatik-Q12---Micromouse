use std::{
    collections::{HashMap, HashSet, VecDeque},
    usize,
};

use tracing::instrument;

use crate::{
    comm::micromouse_message::{
        Command, InterruptAction, InterruptStep, MeasurementInterrupt, MovementType,
        TransformedMovement,
    },
    map::{
        map::{CellDiscoveryStatus, Map, WallDiscoveryStatus},
        world_data::WorldData,
    },
    strategy::strategy::{GoalPosition, StrategyEndState},
    transform::{
        direction::{Direction, DirectionNormalizedVector, RelativeDirection},
        position::{MouseTransform, Position},
    },
    utils::path::Path,
};

// The successor of the DepthFirstBase (Will create a clone of the DFB if the construction is
// successful)
pub struct DepthFirstWithCurrent(DepthFirstBase);

#[derive(Clone, Debug)]
pub struct DepthFirstBase {
    intersections: HashMap<Position, Intersection>,
    intersection_queue: VecDeque<Position>,
    path_from_start: Path,
    current_cmd: TaskExecution,
}

#[derive(Clone, Debug)]
pub struct Intersection {
    visitable_directions: HashSet<Direction>,
}

#[derive(Clone, Debug)]
pub struct IntersectionPath {
    at_intersection: Position,
    in_direction: Direction,
}

#[derive(Clone, Debug)]
pub enum PathRanking {
    Undefined,
    LowestMoves,
    LowestCost {
        turn_value: usize,
        move_value: usize,
    },
}

#[derive(Clone, Debug)]
pub struct TaskExecution {
    from_pos: MouseTransform,
    do_cmd: Command,
}

impl From<TaskExecution> for TransformedMovement {
    fn from(value: TaskExecution) -> Self {
        Self::new(value.do_cmd.ty, value.from_pos)
    }
}

impl DepthFirstBase {
    // Checks whether the current_cmd is completed; Only if it is: returns the DFB with the
    // guarantee, that all new intersections have been added
    pub fn with_current_world<const N: usize>(
        &self,
        world: impl AsRef<WorldData<N>>,
        goal: GoalPosition,
    ) -> Option<DepthFirstWithCurrent> {
        let world_data = world.as_ref();
        let new_intersections =
            self.new_intersection_paths(world_data, Self::is_unvisited_and_open)?;
        let mut successor = self.clone();
        successor.path_from_start.connect_to(world_data.mouse);
        successor.prune_zero_steps(world_data, goal);
        successor.add_new_intersections(new_intersections);
        Some(DepthFirstWithCurrent(successor))
        // todo!("Add the current pos to the path and update the intersection stack if there are any new intersections; Reject if there are not enough measures for the step to be fully complete")
    }

    pub fn add_new_intersections(&mut self, intersection_paths: Vec<IntersectionPath>) {
        for new_path in intersection_paths {
            if let Some(existing_intersection) =
                self.intersections.get_mut(&new_path.at_intersection)
            {
                existing_intersection
                    .visitable_directions
                    .insert(new_path.in_direction);
            } else {
                let pos = new_path.at_intersection;
                let intersection = Intersection {
                    visitable_directions: HashSet::from([new_path.in_direction]),
                };

                self.intersection_queue.push_front(pos);
                self.intersections.insert(pos, intersection);
            }
        }
    }

    pub fn prune_zero_steps<const N: usize>(
        &mut self,
        world: impl AsRef<Map<N>>,
        goal: GoalPosition,
    ) {
        let mut remove_intersections = vec![];
        for (i_pos, i_dirs) in self.intersections.iter_mut() {
            i_dirs.visitable_directions = i_dirs
                .visitable_directions
                .clone()
                .into_iter()
                .filter(|dir| {
                    let from_origin = MouseTransform {
                        pos: *i_pos,
                        dir: *dir,
                    };
                    let max_steps = max_steps_in_direction(&world, from_origin, goal);
                    max_steps != 0
                })
                .collect();
            if i_dirs.visitable_directions.is_empty() {
                remove_intersections.push(*i_pos)
            }
        }
        for remove in &remove_intersections {
            self.intersections.remove(remove);
        }
        self.intersection_queue = self
            .intersection_queue
            .iter()
            .filter(|p| !remove_intersections.contains(p))
            .cloned()
            .collect();
    }

    pub fn new_intersection_paths<const N: usize>(
        &self,
        world: impl AsRef<WorldData<N>>,
        is_visitable: impl Fn(Map<N>, Position, Direction) -> bool,
    ) -> Option<Vec<IntersectionPath>> {
        let world: &WorldData<N> = world.as_ref();
        let transformed_move: TransformedMovement = self.current_cmd.clone().into();

        let mut new_intersections = vec![];

        for step in 0..=transformed_move.max_step_count() {
            let Some(pos_at_step) = transformed_move.at_step(step) else {
                // Reached map walls
                break;
            };

            for rel_dir in [
                RelativeDirection::Left,
                RelativeDirection::Forward,
                RelativeDirection::Right,
            ] {
                let dir = rel_dir.transform_by(&pos_at_step.dir);
                let wall_in_dir = world.map.wall(&pos_at_step.pos, &dir);
                let is_discoverd = match wall_in_dir {
                    None => true,
                    Some(wall) if wall.is_discovered() => true,
                    _ => false,
                };

                if !is_discoverd {
                    return None;
                }

                if is_visitable(world.map, pos_at_step.pos, dir) {
                    let new_path = IntersectionPath {
                        at_intersection: pos_at_step.pos,
                        in_direction: dir,
                    };
                    new_intersections.push(new_path);
                }
            }

            if pos_at_step == world.mouse {
                // Reached an interrupt; Do not have to look further
                break;
            }
        }

        Some(new_intersections)
    }

    #[instrument(
        skip(map),
        name = "is_unvisited_and_open",
        fields(description = "Check whether a given cell is visitable (non visited + inside map)")
    )]
    pub fn is_unvisited_and_open<const N: usize>(
        map: impl Into<Map<N>>,
        from_cell: Position,
        direction: Direction,
    ) -> bool {
        let map: Map<N> = map.into();

        let wall_in_dir = map.wall(&from_cell, &direction);
        if wall_in_dir == Some(&WallDiscoveryStatus::Exists(true)) {
            return false;
        }

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

impl DepthFirstWithCurrent {
    pub fn moves_to_next_intersection(
        &mut self,
        path_ranking: PathRanking,
    ) -> Result<(Vec<MovementType>, MouseTransform), StrategyEndState> {
        let move_back_to_pos =
            self.0
                .intersection_queue
                .front()
                .ok_or(StrategyEndState::NoPossibleAction(
                    "There is no intersection that has not been visited".to_string(),
                ))?;
        let move_back_to_intersection = self
            .0
            .intersections
            .get_mut(move_back_to_pos)
            .expect("Intersection should exist if it was on the stack");
        let possible_directions = &move_back_to_intersection.visitable_directions;
        let mut best_ranking = usize::MAX;
        let mut best_ranked = None;

        for dir in possible_directions {
            let return_to = MouseTransform {
                pos: *move_back_to_pos,
                dir: *dir,
            };
            let moves_to_return = self.0.path_from_start.return_to(return_to).map_err(|p| {
                StrategyEndState::NoPossibleAction(format!(
                    "An intersection on the stack was not part of the path {p:?}"
                ))
            })?;

            let score = match path_ranking {
                PathRanking::Undefined => {
                    return Ok((moves_to_return, return_to));
                }
                PathRanking::LowestMoves => moves_to_return.len(),
                PathRanking::LowestCost {
                    turn_value,
                    move_value,
                } => moves_to_return
                    .iter()
                    .map(|m| match m {
                        MovementType::Turn(x) => x.unsigned_abs() as usize * turn_value,
                        MovementType::Move(x) => *x as usize * move_value,
                    })
                    .sum(),
            };

            if score < best_ranking {
                best_ranking = score;
                best_ranked = Some((moves_to_return, return_to));
            }
        }

        let Some((best_moves, transf_after)) = best_ranked else {
            return Err(StrategyEndState::NoPossibleAction(
                "Intersection was empty".to_string(),
            ));
        };

        move_back_to_intersection
            .visitable_directions
            .remove(&transf_after.dir);
        if move_back_to_intersection.visitable_directions.is_empty() {
            self.0.intersection_queue.pop_front();
        }

        Ok((best_moves, transf_after))

        // todo!("Plot the path back to the next valid intersection (might just mean turning around); Return the transform after these moves are applied")
        // Also: skips any intersections that are guaranteed to have 0 max_steps
    }

    fn move_forward_from<const N: usize>(
        &mut self,
        map: impl AsRef<Map<N>>,
        from_pos: MouseTransform,
        goal: GoalPosition,
    ) -> Command {
        let max_steps_fwd = max_steps_in_direction(map, from_pos, goal);

        let next_move = MovementType::Move(max_steps_fwd as u8);
        let next_cmd = Command {
            ty: next_move,
            interrupts: vec![
                MeasurementInterrupt {
                    direction: RelativeDirection::Forward,
                    at_step: InterruptStep::Each,
                    action: InterruptAction::StopIfBlocked,
                },
                MeasurementInterrupt {
                    direction: RelativeDirection::Left,
                    at_step: InterruptStep::Each,
                    action: InterruptAction::Continue,
                },
                MeasurementInterrupt {
                    direction: RelativeDirection::Right,
                    at_step: InterruptStep::Each,
                    action: InterruptAction::Continue,
                },
            ],
        };

        self.0.current_cmd = TaskExecution {
            from_pos,
            do_cmd: next_cmd.clone(),
        };
        next_cmd
    }

    // Prepares for next step / successor
    // pub fn finish_step(self) -> DepthFirstBase {
    //     self.0
    // }

    // This is the entry-point of the Base <-> Current relationship as this does not require a
    // measure-check before being able to output the initial cmd
    pub fn new<const N: usize>(world: impl AsRef<WorldData<N>>, goal: GoalPosition) -> Self {
        let world = world.as_ref();
        let intersection = Intersection {
            visitable_directions: HashSet::from([
                Direction::PosX,
                Direction::PosY,
                Direction::NegX,
                Direction::NegY,
            ]),
        };
        let mut res = Self(DepthFirstBase {
            intersections: HashMap::from([(world.mouse.pos.clone(), intersection)]),
            intersection_queue: VecDeque::from([world.mouse.pos]),
            path_from_start: Path::new(world.mouse),
            current_cmd: TaskExecution {
                from_pos: world.mouse,
                do_cmd: Command {
                    ty: MovementType::Move(0),
                    interrupts: vec![],
                },
            },
        });

        res.0.prune_zero_steps(world, goal);
        res
    }
}

pub fn max_steps_in_direction<const N: usize>(
    map: impl AsRef<Map<N>>,
    from_origin: MouseTransform,
    goal: GoalPosition,
) -> usize {
    let transf_moves = TransformedMovement::new(MovementType::Move(N as u8), from_origin);

    let map = map.as_ref();

    for i in 0..=N {
        let Some(current) = transf_moves.at_step(i) else {
            // The step is outside the map; should not happen
            return i - 1;
        };
        if current.pos == goal.0 {
            // Does not need to move further
            return i;
        }
        let Some(wall_ahead) = map.wall(&current.pos, &current.dir) else {
            // wall does not exist / is a map boundary --> Is blocking
            return i;
        };
        if *wall_ahead == WallDiscoveryStatus::Exists(true) {
            return i;
        }

        let Some(current_cell) = map.cell(&current.pos) else {
            // The step is outside the map; should not happen
            return i - 1;
        };
        if i != 0 && *current_cell == CellDiscoveryStatus::Visited {
            // We do not need to go this far; The cell after this is already discovered
            return i - 1;
        }
    }
    N
}
