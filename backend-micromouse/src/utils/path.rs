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
            let only_node = self.nodes.get(0).expect("Checked");
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

        let mut current_transf = self.nodes.last().expect("Len >= 1").clone();

        // From the back: delete all the segments until finding one that contains the position (and
        // then also delete that)
        while self.nodes.len() > 1 {
            let last = self.nodes.pop().expect("Len > 1");
            let second_last = self.nodes.last().expect("Len > 1");
            debug!(target: "path/op", "Checking segment {last:?} ..= {second_last:?}");

            let x_range = last.pos.x.min(second_last.pos.x)..=last.pos.x.max(second_last.pos.x);
            let y_range = last.pos.y.min(second_last.pos.y)..=last.pos.y.max(second_last.pos.y);

            let is_inside = x_range.contains(&goal_pos.x) && y_range.contains(&goal_pos.y);
            if is_inside {
                debug!(target: "path/op", "Is inside");
                if second_last.pos == goal_pos {
                    // the goal is on the second_last, just rotated --> The second_last is gonna be
                    // replaced

                    self.nodes.pop();
                    self.nodes.push(goal_on_path);
                } else {
                    let directions_aligned = second_last.dir == goal_on_path.dir;

                    if !directions_aligned {
                        // Inserting an in-between step
                        self.nodes.push(MouseTransform {
                            pos: goal_pos,
                            dir: second_last.dir,
                        });
                    }
                    self.nodes.push(goal_on_path);
                }
                let move_dir = current_transf
                    .pos
                    .direction_straight_line(goal_pos)
                    .expect("Should be in straight line");
                let rotate_to_move = current_transf.dir.shortest_rotate_to(&move_dir);
                if rotate_to_move != 0 {
                    moves.push(MovementType::Turn(rotate_to_move));
                }
                let fwd = last
                    .pos
                    .distance_straight_line(goal_pos)
                    .expect("Should be in straight line");
                moves.push(MovementType::Move(fwd as u8));
                let rotate_to_goal = move_dir.shortest_rotate_to(&goal_on_path.dir);
                if rotate_to_goal != 0 {
                    moves.push(MovementType::Turn(rotate_to_goal));
                }
                break;
            }

            if last.pos != second_last.pos {
                // Move
                let move_dir = last
                    .pos
                    .direction_straight_line(second_last.pos)
                    .expect("Should be in straight line");
                let rotate_to_move = current_transf.dir.shortest_rotate_to(&move_dir);
                if rotate_to_move != 0 {
                    moves.push(MovementType::Turn(rotate_to_move));
                }
                let fwd = last
                    .pos
                    .distance_straight_line(second_last.pos)
                    .expect("Should be in straight line");
                moves.push(MovementType::Move(fwd as u8));
                current_transf = MouseTransform {
                    pos: second_last.pos,
                    dir: move_dir,
                };
            } else {
                // only rotation
                let goal_rot = second_last.dir.rotated(2);
                let rotate = current_transf.dir.shortest_rotate_to(&goal_rot);
                if rotate != 0 {
                    moves.push(MovementType::Turn(rotate))
                }
                current_transf = MouseTransform {
                    pos: second_last.pos,
                    dir: goal_rot,
                };
            }
        }

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

    // Moves one position step towards the root of the path
    // pub fn one_towards_root(&mut self) -> Vec<MovementType> {
    //     if self.nodes.len() <= 1 {
    //         return vec![];
    //     }
    //
    //
    //     let last = self.nodes.pop().expect("Checked");
    //     let second_last = self.nodes.last().expect("Checked");
    //
    //     let mut moves = vec![];
    //     if last.dir == second_last.dir {
    //         // Move
    //         let move_direction = last
    //             .pos
    //             .direction_straight_line(second_last.pos)
    //             .expect("Should be in a line");
    //         let rotate = last.dir.shortest_rotate_to(&move_direction);
    //         if rotate != 0 {
    //             moves.push(MovementType::Turn(rotate))
    //         }
    //
    //         moves.push(MovementType::Move(
    //             last.pos
    //                 .distance_straight_line(second_last.pos)
    //                 .expect("Should be in a line") as u8,
    //         ));
    //     } else {
    //         // Rotation
    //         let rotate = last.dir.shortest_rotate_to(&second_last.dir);
    //         if rotate != 0 {
    //             moves.push(MovementType::Turn(rotate))
    //         }
    //     }
    //
    //     moves
    // }
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
