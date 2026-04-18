use console::Style;

use crate::{
    transform::position::MouseTransform,
    utils::map_display::{MapDisplay, MapDisplayWrite},
};

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
}

pub struct PathReference<'a> {
    map: &'a mut MapDisplay,
    path: Path,
}

impl<'a> MapDisplayWrite for PathReference<'a> {
    fn set_char(&mut self, c: char) {
        for window in self.path.nodes().windows(2) {
            let start = window[0];
            let end = window[1];
            let c = if start.pos.x == end.pos.x {
                '|'
            } else if start.pos.y == end.pos.y {
                '-'
            } else {
                continue;
            };

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
            line.set_char(c);
        }
        for node in self.path.nodes() {
            let Some(mut cell) = self.map.cell_mut(node.pos) else {
                return;
            };
            let mut center = cell.center();
            center.set_char(c);
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
