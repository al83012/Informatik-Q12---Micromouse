use std::{
    collections::{HashMap, HashSet, VecDeque},
    u8, usize,
};

use console::Style;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};

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
    utils::{
        map_display::{MapDisplay, MapDisplayWrite},
        path::{Path, PathReference},
    },
};

// The successor of the DepthFirstBase (Will create a clone of the DFB if the construction is
// successful)
#[derive(Clone, Debug)]
pub struct DepthFirstWithCurrent(DepthFirstBase);

#[derive(Clone, Debug)]
pub struct DepthFirstBase {
    intersections: HashMap<Position, Intersection>,
    intersection_queue: VecDeque<Position>,
    explored: HashSet<IntersectionPath>,
    path_from_start: Path,
    current_cmd: TaskExecution,
}

#[derive(Clone, Debug)]
pub enum MaybeInitDepthFirst {
    HasInitialStep(DepthFirstWithCurrent),
    WithoutCurrentStep(DepthFirstBase),
}

#[derive(Clone, Debug)]
pub struct Intersection {
    visitable_directions: HashSet<Direction>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IntersectionPath {
    at_intersection: Position,
    in_direction: Direction,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize, PartialEq)]
pub enum PathRanking {
    Undefined,
    LowestMoves,
    LowestCost {
        turn_value: usize,
        move_value: usize,
    },
    TowardsGoal,
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
        prune_dead_ends: bool,
    ) -> Option<DepthFirstWithCurrent> {
        let world_data = world.as_ref();
        let new_intersections =
            self.new_intersection_paths(world_data, Self::is_unvisited_and_open)?;
        debug!(target: "strat/dfs", "Strat became expandable: \n{world_data}\nNew intersections: \n{new_intersections:#?}");
        let mut successor = self.clone();
        if !successor.path_from_start.connect_to(world_data.mouse) {
            warn!(target: "strat/dfs", "Failed to connect {:?}\nto {:#?}", world_data.mouse, successor.path_from_start);
        }
        successor.add_new_intersections(new_intersections.clone());
        successor.prune_zero_steps(world_data, goal);

        let mut map_display = MapDisplay::from(&world_data.map);
        let mut path_ref = PathReference::new(&successor.path_from_start, &mut map_display);
        path_ref.set_char('*');
        for explored in successor.explored.iter() {
            let Some(mut x) = map_display.wall_mut(explored.at_intersection, explored.in_direction)
            else {
                continue;
            };
            x.inner().apply_style(Style::new().on_red());
        }
        for (pos, dirs) in successor.intersections.iter() {
            for dir in dirs.visitable_directions.iter() {
                let Some(mut x) = map_display.wall_mut(*pos, *dir) else {
                    continue;
                };
                x.inner().apply_style(Style::new().on_blue());
            }
        }
        for intersection in new_intersections.iter() {
            let Some(mut x) =
                map_display.wall_mut(intersection.at_intersection, intersection.in_direction)
            else {
                continue;
            };
            x.inner().apply_style(Style::new().on_green());
        }

        if prune_dead_ends {
            successor.prune_dead_ends(world_data, goal, Some(&mut map_display));
        }

        if let Some(mut goal) = map_display.cell_mut(goal.0) {
            goal.apply_style(Style::new().on_cyan());
        }

        let map_str = format!("\n{map_display}");
        debug!(target: "strat/dfs", "Added expansion to path: {map_str}");
        Some(DepthFirstWithCurrent(successor))
        // todo!("Add the current pos to the path and update the intersection stack if there are any new intersections; Reject if there are not enough measures for the step to be fully complete")
    }

    pub fn add_new_intersections(&mut self, intersection_paths: Vec<IntersectionPath>) {
        for new_path in intersection_paths {
            debug!(target: "strat/dfs", "ADDING PATH {:?} {:?}", new_path.at_intersection, new_path.in_direction);
            if self.explored.contains(&new_path) {
                debug!(target: "strat/dfs", "   --> Already explored");
                continue;
            }
            if let Some(existing_intersection) =
                self.intersections.get_mut(&new_path.at_intersection)
            {
                debug!(target: "strat/dfs", "   --> Added to existing");
                existing_intersection
                    .visitable_directions
                    .insert(new_path.in_direction);
            } else {
                let pos = new_path.at_intersection;
                let intersection = Intersection {
                    visitable_directions: HashSet::from([new_path.in_direction]),
                };

                self.intersection_queue.push_front(pos);
                debug!(target: "strat/dfs", "   --> Added new as next");
                self.intersections.insert(pos, intersection);
            }
        }
    }

    pub fn prune_zero_steps<const N: usize>(
        &mut self,
        world: impl AsRef<Map<N>>,
        goal: GoalPosition,
    ) {
        debug!(target:"strat/dfs", "Prune zero steps");
        let mut remove_intersections = vec![];
        for (i_pos, i_dirs) in self.intersections.iter_mut() {
            let prune_dirs = i_dirs
                .visitable_directions
                .clone()
                .into_iter()
                .filter(|dir| {
                    let from_origin = MouseTransform {
                        pos: *i_pos,
                        dir: *dir,
                    };
                    let max_steps = max_steps_in_direction(&world, from_origin, goal);
                    max_steps == 0
                });

            for prune_dir in prune_dirs {
                debug!(target:"strat/dfs", "Pruning {i_pos:?} in dir {prune_dir:?}");
                i_dirs.visitable_directions.remove(&prune_dir);
                self.explored.insert(IntersectionPath {
                    at_intersection: *i_pos,
                    in_direction: prune_dir,
                });
            }

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

    pub fn prune_dead_ends<const N: usize>(
        &mut self,
        world: impl AsRef<Map<N>>,
        goal: GoalPosition,
        mut display: Option<&mut MapDisplay>,
    ) {
        let expand_from = goal.0;
        let mut to_expand = vec![expand_from];
        let mut could_access_goal = HashSet::new();

        let map = world.as_ref();

        while let Some(cell_to_expand) = to_expand.pop() {
            if !could_access_goal.insert(cell_to_expand) {
                continue;
            }
            if let Some(ref mut display) = display {
                if let Some(mut cell) = display.cell_mut(cell_to_expand) {
                    cell.apply_style(Style::new().on_blue());
                }
            }
            let neighbor_dirs = vec![
                Direction::PosX,
                Direction::PosY,
                Direction::NegX,
                Direction::NegY,
            ];
            for dir in neighbor_dirs {
                let pos_offset = dir.steps_in_dir(1);
                let Some(neighbor_pos) = cell_to_expand + pos_offset else {
                    continue;
                };
                let wall_to_neighbor = map.wall(&cell_to_expand, &dir);
                if wall_to_neighbor.is_none_or(|wall| {
                    *wall == WallDiscoveryStatus::Exists(true)
                        || *wall == WallDiscoveryStatus::Visited
                }) {
                    continue;
                }
                let neighbor_cell = map.cell(&neighbor_pos);
                if neighbor_cell.is_none_or(|c| *c == CellDiscoveryStatus::Visited) {
                    continue;
                }

                to_expand.push(neighbor_pos);
            }
        }

        let mut remove_intersections = vec![];
        for (i_pos, i_dirs) in self.intersections.iter_mut() {
            let prune_dirs = i_dirs
                .visitable_directions
                .clone()
                .into_iter()
                .filter(|dir| {
                    if let Some(leads_to_dir) = *i_pos + dir.steps_in_dir(1) {
                        !could_access_goal.contains(&leads_to_dir)
                    } else {
                        true
                    }
                });

            for prune_dir in prune_dirs {
                debug!(target:"strat/dfs", "Pruning {i_pos:?} in dir {prune_dir:?} (Cannot access goal)");
                i_dirs.visitable_directions.remove(&prune_dir);
                let intersection_path = IntersectionPath {
                    at_intersection: *i_pos,
                    in_direction: prune_dir,
                };
                self.explored.insert(intersection_path.clone());
                if let Some(ref mut display) = display {
                    if let Some(mut w) = display.wall_mut(*i_pos, prune_dir) {
                        w.apply_style(Style::new().on_magenta());
                    }
                }
            }

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
        info!(target: "strat/dfs", "TESTING EXPANDABLE after doing move {:?}", self.current_cmd);
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
                let is_discovered = match wall_in_dir {
                    None => true,
                    Some(wall) if wall.is_discovered() => true,
                    _ => false,
                };

                if !is_discovered {
                    info!(target: "strat/dfs", "Failed at {pos_at_step:?} {dir:?} --> Undiscovered");
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
        if wall_in_dir.is_none() {
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
        goal: GoalPosition,
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
        info!(target: "strat/dfs", "NEXT INTERSECTION = {move_back_to_pos:?}");
        let possible_directions = &move_back_to_intersection.visitable_directions;
        let mut best_ranking = usize::MAX;
        let mut best_ranked = None;

        for dir in possible_directions {
            let return_to = MouseTransform {
                pos: *move_back_to_pos,
                dir: *dir,
            };
            let moves_to_return = self
                .0
                .path_from_start
                .clone()
                .return_to(return_to)
                .map_err(|p| {
                    StrategyEndState::NoPossibleAction(format!(
                        "An intersection on the stack was not part of the path {p:?}"
                    ))
                })?;

            let score = match path_ranking {
                PathRanking::Undefined => {
                    best_ranked = Some((moves_to_return, return_to));
                    break;
                    // return Ok((moves_to_return, return_to));
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
                PathRanking::TowardsGoal => {
                    if let Some(neighbor_pos) = *move_back_to_pos + dir.steps_in_dir(1) {
                        let dx = goal.0.x as i32 - neighbor_pos.x as i32;
                        let dy = goal.0.y as i32 - neighbor_pos.y as i32;

                        (dx * dx + dy * dy) as usize
                    } else {
                        usize::MAX
                    }
                }
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

        self.0
            .path_from_start
            .return_to(transf_after)
            .expect("Previously checked");

        move_back_to_intersection
            .visitable_directions
            .remove(&transf_after.dir);
        if move_back_to_intersection.visitable_directions.is_empty() {
            let removed_pos = self
                .0
                .intersection_queue
                .pop_front()
                .expect("Was just in there");
            self.0.intersections.remove(&removed_pos);
        }

        info!(target: "strat/dfs", "After known moves: \n{:#?}", self.0.path_from_start.nodes());

        Ok((best_moves, transf_after))

        // todo!("Plot the path back to the next valid intersection (might just mean turning around); Return the transform after these moves are applied")
        // Also: skips any intersections that are guaranteed to have 0 max_steps
    }

    pub fn move_forward_from<const N: usize>(
        mut self,
        map: impl AsRef<Map<N>>,
        from_pos: MouseTransform,
        goal: GoalPosition,
        interrupt_right: bool,
        interrupt_left: bool,
    ) -> (Command, DepthFirstBase) {
        let max_steps_fwd = max_steps_in_direction(map, from_pos, goal);

        self.0.explored.insert(IntersectionPath {
            at_intersection: from_pos.pos,
            in_direction: from_pos.dir,
        });
        let next_move = MovementType::Move(max_steps_fwd as u8);
        let side_interrupts = (1..=max_steps_fwd).flat_map(|i| {
            let left_i = if interrupt_left {
                Some(MeasurementInterrupt {
                    direction: RelativeDirection::Left,
                    at_step: InterruptStep::At(i as u32),
                    action: InterruptAction::StopIfOpen,
                })
            } else {
                None
            };
            let right_i = if interrupt_right {
                Some(MeasurementInterrupt {
                    direction: RelativeDirection::Right,
                    at_step: InterruptStep::At(i as u32),
                    action: InterruptAction::StopIfOpen,
                })
            } else {
                None
            };
            left_i.into_iter().chain(right_i)
        });
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
            ]
            .into_iter()
            .chain(side_interrupts)
            .collect(),
        };

        self.0.current_cmd = TaskExecution {
            from_pos,
            do_cmd: next_cmd.clone(),
        };
        (next_cmd, self.0)
    }

    // Prepares for next step / successor
    // pub fn finish_step(self) -> DepthFirstBase {
    //     self.0
    // }

    // This is the entry-point of the Base <-> Current relationship as this does not require a
    // measure-check before being able to output the initial cmd
    pub fn new<const N: usize>(world: impl AsRef<WorldData<N>>) -> Self {
        let world = world.as_ref();
        let intersection = Intersection {
            visitable_directions: HashSet::from([
                Direction::PosX,
                Direction::PosY,
                Direction::NegX,
                Direction::NegY,
            ]),
        };
        let res = Self(DepthFirstBase {
            intersections: HashMap::from([(world.mouse.pos, intersection)]),
            explored: HashSet::new(),
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

    info!(target: "strat/dfs", "Checking len from {transf_moves:?}");

    for i in 0..=N {
        let Some(current) = transf_moves.at_step(i) else {
            error!(target: "strat/dfs", "Outside map at {i}");
            // The step is outside the map; should not happen
            return i - 1;
        };
        if current.pos == goal.0 {
            debug!(target: "strat/dfs", " --> Stopping on goal at {i}: {current:?}");
            // Does not need to move further
            return i;
        }
        let Some(current_cell) = map.cell(&current.pos) else {
            // The step is outside the map; should not happen
            error!(target: "strat/dfs", " --> Encountered cell outside bounds --> Should have stopped at {}", i - 1);
            return i - 1;
        };
        if i != 0 && *current_cell == CellDiscoveryStatus::Visited {
            debug!(target: "strat/dfs", " --> Encountered visited --> Should have stopped at {}", i - 1);
            // We do not need to go this far; The cell after this is already discovered
            return i - 1;
        }
        let Some(wall_ahead) = map.wall(&current.pos, &current.dir) else {
            debug!(target: "strat/dfs", " --> Hit map boundary --> Stopping at {i}: {current:?}");
            // wall does not exist / is a map boundary --> Is blocking
            return i;
        };
        if *wall_ahead == WallDiscoveryStatus::Exists(true) {
            debug!(target: "strat/dfs", " --> Hit wall --> Stopping at {i}: {current:?}");
            return i;
        }
    }
    debug!(target: "strat/dfs", "ALLOWED FULL LEN {N}");
    N
}
