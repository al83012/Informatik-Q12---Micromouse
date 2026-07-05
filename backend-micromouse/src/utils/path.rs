use console::Style;
use tracing::{debug, error};

use crate::{
    comm::micromouse_message::{MovementType, TransformedMovement},
    transform::{
        direction::{Direction, DirectionNormalizedVector},
        position::{MouseTransform, Position},
    },
    utils::map_display::{MapDisplay, MapDisplayWrite},
};

#[derive(Clone, Debug)]
pub struct Path {
    nodes: Vec<MouseTransform>,
}

pub struct VisualPath {
    path: Path,
    style: Style,
}

impl Path {
    pub fn new(start: MouseTransform) -> Self {
        Path { nodes: vec![start] }
    }

    pub fn start(&self) -> &MouseTransform {
        self.nodes.first().expect("Len >= 1")
    }

    pub fn last(&self) -> &MouseTransform {
        self.nodes.last().expect("len >= 1")
    }

    pub fn connect_to(&mut self, next_node: MouseTransform) -> bool {
        let last = self
            .nodes
            .last()
            .expect("Construction guarantees at least one element");
        // Either the last x or the last y has to be equal
        let last_x_eq = last.pos.x == next_node.pos.x;
        let last_y_eq = last.pos.y == next_node.pos.y;
        let last_dir_eq = last.dir == next_node.dir;

        if ((last_x_eq != last_y_eq) && last_dir_eq) || (last_x_eq && last_y_eq && !last_dir_eq) {
            // One axis has changed, everything else is equal
            self.nodes.push(next_node);
            true
        } else {
            false
        }
    }

    // Like connect_to, but will first check that the given end is not already the last element
    pub fn end_with(&mut self, end: MouseTransform) -> bool {
        if self.nodes.last() != Some(&end) {
            self.connect_to(end)
        } else {
            false
        }
    }

    pub fn nodes(&self) -> &Vec<MouseTransform> {
        &self.nodes
    }

    /// Will try to chart a path back to the given Position (if it is on the path), which is valid
    /// without knowing the map, as it is assumed that the path is walkable
    pub fn return_to(
        &mut self,
        goal_on_path: MouseTransform,
    ) -> Result<Vec<MovementType>, PathTraversalError> {
        debug!(target: "path/op", "Return to {goal_on_path:?} for {:#?}", self.nodes);
        if self.nodes.len() == 1 {
            debug!(target: "path/op", "Only 1 node");
            let only_node = self.nodes.pop().expect("Checked");
            if only_node.pos == goal_on_path.pos {
                debug!(target: "path/op", "Same pos");
                let rotate = only_node.dir.shortest_rotate_to(&goal_on_path.dir);
                return Ok(if rotate != 0 {
                    debug!(target: "path/op", "Adding rotation");
                    // THEORETICALLY EXPANDING the path
                    self.nodes.push(goal_on_path);
                    vec![MovementType::Turn(rotate)]
                } else {
                    debug!(target: "path/op", "Same dir");
                    vec![]
                });
            } else {
                debug!(target: "path/op", "Not on path");
                return Err(PathTraversalError::PositionNotOnPath);
            }
        }
        let goal_pos = goal_on_path.pos;
        if !self.contains(&goal_pos) {
            debug!(target: "path/op", "Not on path");
            return Err(PathTraversalError::PositionNotOnPath);
        }

        let mut moves = vec![];
        let mut current_transf = *self.nodes.last().expect("len > 1");

        //INFO: Searching the right segment

        // The directions in which the path entered a cell; not the rotation within
        let mut cell_entrance_directions: Vec<(usize, MouseTransform)> = vec![];
        for (node_id, node) in self.nodes.iter().enumerate() {
            if cell_entrance_directions.last().map(|(_id, pos)| pos.pos) != Some(node.pos) {
                cell_entrance_directions.push((node_id, *node));
            }
        }

        debug!(target: "path", "Cell entrances: \n{cell_entrance_directions:#?}");

        let mut goal_before_entrance_id = None;

        for i in (1..cell_entrance_directions.len()).rev() {
            let (maybe_before_id, maybe_before_node) = cell_entrance_directions[i];
            let (maybe_after_or_on_id, to_node) = cell_entrance_directions[i - 1];
            debug!(target: "path", "Checking entrance segment {maybe_before_id}..={maybe_after_or_on_id}");

            let dir_to_node = maybe_before_node
                .pos
                .direction_straight_line(to_node.pos)
                .expect("Should be in straight line");
            let without_before = (maybe_before_node.pos + dir_to_node.steps_in_dir(1))
                .expect("Should be a valid pos");

            let pos_x_range =
                without_before.x.min(to_node.pos.x)..=without_before.x.max(to_node.pos.x);
            let pos_y_range =
                without_before.y.min(to_node.pos.y)..=without_before.y.max(to_node.pos.y);

            debug!(target: "path", "X: {pos_x_range:?}, Y: {pos_y_range:?}");

            if pos_x_range.contains(&goal_pos.x) && pos_y_range.contains(&goal_pos.y) {
                goal_before_entrance_id = Some(i);
                break;
            }
        }
        debug!(target: "path", "Goal before entrance #{goal_before_entrance_id:?}");

        let Some(goal_before_entrance_id) = goal_before_entrance_id else {
            debug!(target: "path", "--> Directly on last cell");
            let rotate_to = goal_on_path.dir;
            let entrance = cell_entrance_directions.last().expect("len > 1");
            let entrance_rotation = entrance.1.dir;
            let entrance_idx = entrance.0;
            let final_rotation = current_transf.dir;

            // remove up to the entrance:
            while self.nodes.len() > entrance_idx + 1 {
                let removed = self.nodes.pop();
                debug!(target: "path", "Removed: {removed:?}");
            }

            if rotate_to != entrance_rotation {
                // Only if the goal actually changes sth
                self.nodes.push(goal_on_path);
                debug!(target: "path", "Added: {goal_on_path:?}");
            }

            let rotation_cmd = final_rotation.shortest_rotate_to(&rotate_to);
            if rotation_cmd != 0 {
                debug!(target: "path", " --> Rotated {rotation_cmd}");
                moves.push(MovementType::Turn(rotation_cmd));
            }

            return Ok(moves);
        };

        let rotate_to = cell_entrance_directions
            .last()
            .expect("len > 1")
            .1
            .dir
            .rotated(2);

        let current_rotate = current_transf.dir;

        let initial_rotation = current_rotate.shortest_rotate_to(&rotate_to);
        if initial_rotation != 0 {
            debug!(target: "path", " --> Turned around {initial_rotation}");
            moves.push(MovementType::Turn(initial_rotation));
        }
        debug!(target: "path", " --> New dir = {rotate_to}");
        current_transf.dir = rotate_to;

        // INFO: Now the micromouse would be turned around to the opposite of the last entrance

        for window in cell_entrance_directions.windows(2).rev() {
            let mut window = window.iter();
            let (move_to_id, prev_entrance) = window.next().expect("Window size > 0");
            let (move_from_id, current_entrance) = window.next().expect("Window size > 1");
            debug!(target: "path", "Processing segment ({prev_entrance:?} #{move_to_id} <-- {current_entrance:?} #{move_from_id})");
            if *move_from_id == cell_entrance_directions[goal_before_entrance_id].0 {
                debug!(target: "path", "At entrance after goal ({:?})", current_transf.pos);
                // The goal is on the next segment
                break;
            }

            let move_to_pos = prev_entrance.pos;
            let current_pos = current_transf.pos;

            let movement_len = current_pos
                .distance_straight_line(move_to_pos)
                .expect("Should be in straight line");
            if movement_len != 0 {
                debug!(target: "path", " --> Moved {movement_len}");
                moves.push(MovementType::Move(movement_len as u8));
            }

            current_transf.pos = move_to_pos;

            let rotate_to_dir = prev_entrance.dir.rotated(2);
            let current_dir = current_transf.dir;

            let rotation = current_dir.shortest_rotate_to(&rotate_to_dir);
            if rotation != 0 {
                debug!(target: "path", " --> Rotated {rotation}");
                moves.push(MovementType::Turn(rotation));
            }

            current_transf.dir = rotate_to_dir;
        }

        // INFO: Move forward to goal:
        let move_to_pos = goal_pos;
        let current_pos = current_transf.pos;
        let movement_len = current_pos
            .distance_straight_line(move_to_pos)
            .expect("Should be in straight line");
        if movement_len != 0 {
            debug!(target: "path", "Last move to goal: {movement_len}");
            moves.push(MovementType::Move(movement_len as u8));
        }

        let rotate_to_dir = goal_on_path.dir;
        let current_dir = current_transf.dir;

        let rotation = current_dir.shortest_rotate_to(&rotate_to_dir);
        if rotation != 0 {
            debug!(target: "path", "Last rotate to goal: {rotation}");
            moves.push(MovementType::Turn(rotation));
        }

        let (directly_after_entrance_node_id, directly_after_entrance_transf) =
            cell_entrance_directions[goal_before_entrance_id - 1];

        while self.nodes.len() > directly_after_entrance_node_id + 1 {
            self.nodes.pop();
        }

        debug!(target: "path", "Current nodes: {:#?}", self.nodes);
        if directly_after_entrance_transf.pos != goal_pos {
            // Adding an exit from the entrance to the goal
            let exit_direction = directly_after_entrance_transf
                .pos
                .direction_straight_line(goal_pos)
                .expect("Should be in straight line");
            debug!(target: "path", "Adding new exit {exit_direction}");
            self.nodes.push(MouseTransform {
                pos: directly_after_entrance_transf.pos,
                dir: exit_direction,
            });
            self.nodes.push(MouseTransform {
                pos: goal_pos,
                dir: exit_direction,
            });
        }

        let last_path_dir = self.nodes.last().expect("len > 1").dir;
        let goal_dir = goal_on_path.dir;

        if last_path_dir != goal_dir {
            debug!(target: "path", "Adding goal: {goal_on_path:?}");
            self.nodes.push(goal_on_path);
        }

        let mut move_to_combine = None;
        let mut combined_moves = vec![];
        for movement in moves.into_iter() {
            if let Some(mc) = move_to_combine {
                match (movement, mc) {
                    (MovementType::Turn(t1), MovementType::Turn(t2)) => {
                        move_to_combine = Some(MovementType::Turn(t1 + t2))
                    }
                    (MovementType::Move(m1), MovementType::Move(m2)) => {
                        move_to_combine = Some(MovementType::Move(m1 + m2))
                    }
                    _ => {
                        if mc.max_step_count() > 0 {
                            combined_moves.push(mc);
                        }
                        move_to_combine = Some(movement);
                    }
                }
            } else {
                move_to_combine = Some(movement);
            }
        }

        if let Some(move_to_combine) = move_to_combine {
            if move_to_combine.max_step_count() > 0 {
                combined_moves.push(move_to_combine);
            }
        }

        Ok(combined_moves)
    }

    pub fn contains(&self, position: &Position) -> bool {
        for [n1, n2] in self.nodes.as_slice().array_windows() {
            let pos_x_range = n1.pos.x.min(n2.pos.x)..=n1.pos.x.max(n2.pos.x);
            let pos_y_range = n1.pos.y.min(n2.pos.y)..=n1.pos.y.max(n2.pos.y);

            if !(pos_x_range.contains(&position.x) && pos_y_range.contains(&position.y)) {
                // Goal is not on this line segment
                continue;
            }
            return true;
        }
        false
    }

    pub fn reversed(&self) -> Self {
        Self {
            nodes: self
                .nodes
                .clone()
                .into_iter()
                .rev()
                .map(|t| t.rotated(2))
                .collect(),
        }
    }

    pub fn one_towards_destination(&mut self) -> Option<MovementType> {
        if self.nodes.len() <= 1 {
            return None;
        }

        let first = self.nodes.remove(0);
        let second = self.nodes.first().expect("Checked");

        if first.dir == second.dir {
            // Move
            Some(MovementType::Move(
                first
                    .pos
                    .distance_straight_line(second.pos)
                    .expect("Should be in a line") as u8,
            ))
        } else {
            // Rotation
            let rotate = first.dir.shortest_rotate_to(&second.dir);
            Some(MovementType::Turn(rotate))
        }
    }

    // TODO: Test reduction
    pub fn reduced(self) -> Self {
        let mut new_nodes = vec![*self.nodes.first().expect("Len >= 1")];
        let mut current_move = MovementType::Turn(0);

        for node_window in self.nodes().windows(2) {
            let from_node = node_window[0];
            let to_node = node_window[1];

            debug!(target: "path", "{from_node:?}..{to_node:?}");

            let movement = if from_node.pos == to_node.pos {
                MovementType::Turn(from_node.dir.shortest_rotate_to(&to_node.dir))
            } else {
                MovementType::Move(
                    from_node
                        .pos
                        .distance_straight_line(to_node.pos)
                        .expect("In straight line") as u8,
                )
            };

            match (&mut current_move, movement) {
                (MovementType::Turn(acc), MovementType::Turn(add)) => *acc += add,
                (MovementType::Move(acc), MovementType::Move(add)) => *acc += add,
                (prev_acc, new) => {
                    debug!(target: "path", "--> {prev_acc:?} +/= {new:?}");
                    let from_cell = new_nodes.last().expect("Len >= 1");
                    let transf = TransformedMovement::new(*prev_acc, *from_cell);
                    let new_step = transf.at_step(transf.max_step_count()).expect("Checked");
                    if new_step != *from_cell {
                        debug!(target: "path", "Step did something");
                        new_nodes.push(new_step);
                    }
                    current_move = new;
                }
            }
        }
        if current_move.max_step_count() > 0 {
            let from_cell = new_nodes.last().expect("Len >= 1");
            let transf = TransformedMovement::new(current_move, *from_cell);
            new_nodes.push(transf.at_step(transf.max_step_count()).expect("Checked"));
        }
        Self { nodes: new_nodes }
    }

    pub fn required_openings(&self) -> Vec<MouseTransform> {
        let mut openings = vec![];
        for adjacent in self.nodes.windows(2) {
            let from = adjacent[0];
            let to = adjacent[1];

            if from.pos != to.pos && from.dir == to.dir {
                let dir = from.dir;
                let dist = from
                    .pos
                    .distance_straight_line(to.pos)
                    .expect("Checked in straight line");
                openings.append(
                    &mut ((0..dist).filter_map(|d| {
                        (from.pos + dir.steps_in_dir(d as u8)).map(|from_cell| MouseTransform {
                            pos: from_cell,
                            dir,
                        })
                    }))
                    .collect(),
                );
            }
        }
        openings
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PathTraversalError {
    PositionNotOnPath,
    OnlyOneNode,
}

pub struct PathReference<'a> {
    map: &'a mut MapDisplay,
    path: &'a Path,
}

impl<'a> MapDisplayWrite for PathReference<'a> {
    fn set_char(&mut self, c: char) {
        debug!(target: "test/display", "set_char");
        for window in self.path.nodes().windows(2) {
            let start = window[0];
            let end = window[1];
            debug!(target: "test/display", "set_char segment {start:?}->{end:?}");
            if start.pos == end.pos {
                debug!(target: "test/display", "Rotation");
                continue;
            }
            let c = if start.pos.x == end.pos.x {
                debug!(target: "test/display", "Vertical");
                '|'
            } else if start.pos.y == end.pos.y {
                debug!(target: "test/display", "Horizontal");
                '-'
            } else {
                error!(target: "test/display", "UNKNOWN");
                continue;
            };

            let Some(start_char_range) = self.map.inner_cell_char_range(start.pos) else {
                error!(target: "test/display", "Invalid start pos");
                return;
            };
            debug!(target: "test/display", "StartCell: {start_char_range:?}");
            let Some(end_char_range) = self.map.inner_cell_char_range(end.pos) else {
                error!(target: "test/display", "Invalid end pos");
                return;
            };
            debug!(target: "test/display", "EndCell: {end_char_range:?}");
            let start_center = start_char_range.center_point();
            let end_center = end_char_range.center_point();
            let Some(mut line) = self.map.line(start_center, end_center) else {
                error!(target: "test/display", "Invalid line ref");
                return;
            };
            debug!(target: "test/display", "Line: {}", line.line_display());
            line.set_char(c);
        }
        for node in self.path.nodes() {
            let Some(mut cell) = self.map.cell_mut(node.pos) else {
                error!(target: "test/display", "Cannot reference cell");
                return;
            };
            let mut center = cell.center();
            center.set_char(c);

            let center_offset: DirectionNormalizedVector = node.dir.into();
            let mut offset_center = center
                .shift(center_offset.y as isize, center_offset.x as isize)
                .expect("Should be in range");

            offset_center.set_char(match node.dir {
                Direction::PosX => '>',
                Direction::PosY => 'V',
                Direction::NegX => '<',
                Direction::NegY => 'A',
            });
        }
    }
    fn apply_style(&mut self, style: Style) {
        for window in self.path.nodes().windows(2) {
            let start = window[0];
            let end = window[1];

            let Some(start_char_range) = self.map.inner_cell_char_range(start.pos) else {
                return;
            };
            let Some(end_char_range) = self.map.inner_cell_char_range(end.pos) else {
                return;
            };
            let start_center = start_char_range.center_point();
            let end_center = end_char_range.center_point();
            let Some(mut line) = self.map.line(start_center, end_center) else {
                return;
            };
            line.apply_style(style.clone());
        }
    }
}

impl<'a> PathReference<'a> {
    pub fn new(path: &'a Path, map: &'a mut MapDisplay) -> Self {
        Self { map, path }
    }
}
