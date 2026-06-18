use console::Style;
use tracing::{debug, error};

use crate::{
    comm::micromouse_message::MovementType,
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
        debug!(target: "path/op", "Return to {goal_on_path:?}");
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

        //INFO: Searching the right segment

        // The directions in which the path entered a cell; not the rotation within
        let mut cell_entrance_directions: Vec<(usize, MouseTransform)> = vec![];
        for (node_id, node) in self.nodes.iter().enumerate() {
            if cell_entrance_directions.last().map(|(_id, pos)| pos.pos) != Some(node.pos) {
                cell_entrance_directions.push((node_id, *node));
            }
        }

        let mut goal_before_entrance = None;

        for i in (1..cell_entrance_directions.len()).rev() {
            let (maybe_before_id, maybe_before_node) = cell_entrance_directions[i];
            let (_, to_node) = cell_entrance_directions[i - 1];

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

            if pos_x_range.contains(&goal_pos.x) && pos_y_range.contains(&goal_pos.y) {
                goal_before_entrance = Some(i);
            }
        }

        let Some(goal_before_entrance) = goal_before_entrance else {
            todo!("Goal is on last pos --> Remove up to the entrance + just rotate + add new (or not, if it is aligned with the entrance)");
            todo!("Return");
        };
        todo!("reverse relative to last entrance");
        todo!("Likewise --> move from 1 entrance to the other (in reverse) until reaching the entrance that is right after the goal");
        todo!("(Delete all nodes on the way there; Add the respective moves)");

        todo!("Once reaching the entrance after the goal: Move forward until reaching the goal");
        todo!("Then: rotate to the right direction");
        todo!("Delete all nodes after the entrance right before the goal, then place the goal in there");

        Ok(moves)
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

    pub fn one_towards_destination(&mut self) -> Vec<MovementType> {
        if self.nodes.len() <= 1 {
            return vec![];
        }

        let first = self.nodes.remove(0);
        let second = self.nodes.first().expect("Checked");

        let mut moves = vec![];
        if first.dir == second.dir {
            // Move
            moves.push(MovementType::Move(
                first
                    .pos
                    .distance_straight_line(second.pos)
                    .expect("Should be in a line") as u8,
            ));
        } else {
            // Rotation
            let rotate = first.dir.shortest_rotate_to(&second.dir);
            if rotate != 0 {
                moves.push(MovementType::Turn(rotate))
            }
        }

        moves
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PathTraversalError {
    PositionNotOnPath,
    OnlyOneNode,
}

pub struct PathReference<'a> {
    map: &'a mut MapDisplay,
    path: Path,
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
    pub fn new(path: Path, map: &'a mut MapDisplay) -> Self {
        Self { map, path }
    }
}
