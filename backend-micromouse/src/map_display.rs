use std::{
    io::Write,
    ops::{Range, RangeInclusive},
};

use console::{style, Style};
use tokio::io::stdout;
use tracing::{debug, trace};

use crate::{
    direction::{self, Direction},
    map::{CellDiscoveryStatus, Map, WallDiscoveryStatus},
    position::{self, Position},
};

pub struct MapDisplay {
    cell_width: usize,
    cell_height: usize,
    // ordered as [row/line][col]
    lines: Vec<Vec<char>>,
    paint: Vec<Vec<Style>>,
    map_size: usize,
}

/// ONLY the inner part of the cell, not the walls
pub struct CellReference<'a> {
    map: &'a mut MapDisplay,
    position: Position,
}

pub struct WallReference<'a> {
    map: &'a mut MapDisplay,
    position: Position,
    direction: Direction,
}

pub struct CharRangeReference<'a> {
    map: &'a mut MapDisplay,
    range: CharRange,
}

#[derive(Debug)]
pub struct CharRange {
    pub range_row: RangeInclusive<usize>,
    pub range_col: RangeInclusive<usize>,
}

pub struct CharPos {
    pub row: usize,
    pub col: usize,
}

pub trait MapDisplayWrite {
    fn apply_style(&mut self, style: Style);
    fn set_char(&mut self, c: char);
}

impl MapDisplay {
    pub fn new(map_size: usize, cell_width: usize, cell_height: usize) -> Self {
        let char_width = (cell_width - 1) * map_size + 1;
        let char_height = (cell_height - 1) * map_size + 1;
        let mut lines = Vec::with_capacity(char_width);
        let mut paint = Vec::with_capacity(char_width);
        for _row in 0..char_height {
            let mut line = Vec::with_capacity(char_width);
            let mut paint_line = Vec::with_capacity(char_width);
            for _col in 0..char_width {
                line.push(' ');
                paint_line.push(Style::default());
            }
            lines.push(line);
            paint.push(paint_line);
        }
        Self {
            lines,
            map_size,
            cell_width,
            cell_height,
            paint,
        }
    }

    pub fn map_size(&self) -> usize {
        self.map_size
    }

    fn upper_left_wall_corner(&self, position: Position) -> Option<CharPos> {
        if self.pos_invalid(position) {
            return None;
        }
        Some(CharPos {
            row: (self.cell_height - 1) * position.y as usize,
            col: (self.cell_width - 1) * position.x as usize,
        })
    }

    fn pos_invalid(&self, position: Position) -> bool {
        position.x as usize > self.map_size + 1 || position.y as usize > self.map_size + 1
    }

    pub fn inner_cell_char_range(&self, position: Position) -> Option<CharRange> {
        let upper_left_wall_corner = self.upper_left_wall_corner(position)?;
        let upper_left_inner_cell_corner = CharPos {
            row: upper_left_wall_corner.row + 1,
            col: upper_left_wall_corner.col + 1,
        };
        let lower_right_inner_cell_corner = CharPos {
            row: upper_left_wall_corner.row + self.cell_height - 2,
            col: upper_left_wall_corner.col + self.cell_width - 2,
        };

        let char_range = CharRange {
            range_row: upper_left_inner_cell_corner.row..=lower_right_inner_cell_corner.row,
            range_col: upper_left_inner_cell_corner.col..=lower_right_inner_cell_corner.col,
        };

        trace!(target: "tests/map/display", "Inner cell &({char_range:?})");
        Some(char_range)
    }

    pub fn cell_mut<'a>(&'a mut self, position: Position) -> Option<CellReference<'a>> {
        if self.pos_invalid(position) {
            return None;
        }
        trace!(target: "tests/map/display", "Cell &({position:?})");
        Some(CellReference {
            map: self,
            position,
        })
    }

    pub fn wall_mut<'a>(
        &'a mut self,
        position: Position,
        direction: Direction,
    ) -> Option<WallReference<'a>> {
        if self.pos_invalid(position) {
            return None;
        }
        trace!(target: "tests/map/display", "Wall &({position:?} {direction:?})");
        Some(WallReference {
            map: self,
            position,
            direction,
        })
    }

    pub fn apply_style(&mut self, char_row: usize, char_col: usize, style: Style) {
        if !(char_row < self.lines.len() && char_col < self.lines[0].len()) {
            return;
        }

        self.paint[char_row][char_col] = style;
    }
    pub fn set_char(&mut self, char_row: usize, char_col: usize, c: char) {
        if !(char_row < self.lines.len() && char_col < self.lines[0].len()) {
            return;
        }

        self.lines[char_row][char_col] = c;
    }
}

impl<'b> CellReference<'b> {
    pub fn all<'a>(&'a mut self) -> CharRangeReference<'a> {
        let range = self
            .map
            .inner_cell_char_range(self.position)
            .expect("Already checked at construction");
        CharRangeReference {
            map: self.map,
            range,
        }
    }
    pub fn center<'a>(&'a mut self) -> CharRangeReference<'a> {
        let range = self
            .map
            .inner_cell_char_range(self.position)
            .expect("Already checked at construction");
        let range_row = range.range_row;
        let range_col = range.range_col;
        let center_row = (range_row.start() + range_row.end()) / 2;
        let center_col = (range_col.start() + range_col.end()) / 2;
        CharRangeReference {
            map: self.map,
            range: CharRange {
                range_row: center_row..=center_row,
                range_col: center_col..=center_col,
            },
        }
    }
}

impl<'b> WallReference<'b> {
    pub fn full<'a>(&'a mut self) -> CharRangeReference<'a> {
        let inner_range = self
            .map
            .inner_cell_char_range(self.position)
            .expect("Already checked at construction");
        let (range_row, range_col) = match self.direction {
            Direction::PosY => {
                // down --> row-
                let wall_row = inner_range.range_row.end() + 1;
                let extended_col =
                    inner_range.range_col.start() - 1..=inner_range.range_col.end() + 1;
                ((wall_row..=wall_row), extended_col)
            }
            Direction::NegX => {
                // left --> col-
                let wall_col = inner_range.range_col.start() - 1;
                let extended_row =
                    inner_range.range_row.start() - 1..=inner_range.range_row.end() + 1;
                (extended_row, (wall_col..=wall_col))
            }
            Direction::NegY => {
                // up --> row+
                let wall_row = inner_range.range_row.start() - 1;
                let extended_col =
                    inner_range.range_col.start() - 1..=inner_range.range_col.end() + 1;
                ((wall_row..=wall_row), extended_col)
            }
            Direction::PosX => {
                // right --> col+
                let wall_col = inner_range.range_col.end() + 1;
                let extended_row =
                    inner_range.range_row.start() - 1..=inner_range.range_row.end() + 1;
                (extended_row, (wall_col..=wall_col))
            }
        };
        CharRangeReference {
            map: self.map,
            range: CharRange {
                range_row,
                range_col,
            },
        }
    }
    pub fn inner<'a>(&'a mut self) -> CharRangeReference<'a> {
        let direction = self.direction;
        let full = self.full();
        match direction {
            Direction::PosX | Direction::NegX => {
                //Wall is vertical --> reduce row
                let range_row = full.range.range_row;
                trace!(target: "tests/map/display", "Range row reduction row = {range_row:?}, col = {:?}", full.range.range_col);
                CharRangeReference {
                    map: full.map,
                    range: CharRange {
                        range_row: range_row.start() + 1..=range_row.end() - 1,
                        range_col: full.range.range_col,
                    },
                }
            }
            Direction::PosY | Direction::NegY => {
                //Wall is horizontal --> reduce col
                let range_col = full.range.range_col;
                trace!(target: "tests/map/display", "Range row reduction row = {:?}, col = {range_col:?}", full.range.range_row);
                CharRangeReference {
                    map: full.map,
                    range: CharRange {
                        range_row: full.range.range_row,
                        range_col: range_col.start() + 1..=range_col.end() - 1,
                    },
                }
            }
        }
    }
}

impl<'a> MapDisplayWrite for WallReference<'a> {
    fn apply_style(&mut self, style: Style) {
        self.full().apply_style(style);
    }

    fn set_char(&mut self, c: char) {
        self.full().set_char(c);
    }
}

impl<'a> MapDisplayWrite for CharRangeReference<'a> {
    fn set_char(&mut self, c: char) {
        for row in self.range.range_row.clone() {
            for col in self.range.range_col.clone() {
                self.map.set_char(row, col, c);
            }
        }
    }

    fn apply_style(&mut self, style: Style) {
        for row in self.range.range_row.clone() {
            for col in self.range.range_col.clone() {
                self.map.apply_style(row, col, style.clone());
            }
        }
    }
}

impl<'a> MapDisplayWrite for CellReference<'a> {
    fn apply_style(&mut self, style: Style) {
        self.all().apply_style(style);
    }

    fn set_char(&mut self, c: char) {
        self.all().set_char(c);
    }
}

impl std::fmt::Display for MapDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug!(target: "tests/map/display", "Displaying MapDisplay");
        for (row_num, row) in self.lines.iter().enumerate() {
            let mut row_str = "".to_string();
            for (col_num, c) in row.iter().enumerate() {
                let style = &self.paint[row_num][col_num];
                row_str.push_str(style.apply_to(c).to_string().as_str());

                // write!(f, "{}", style.apply_to(c))?;
            }
            f.write_str(&row_str)?;
            // std::io::stdout().write_all(row_str.as_bytes()).expect("errored");
            // std::io::stdout().write_all("\n".as_bytes()).expect("errored");
            // std::io::stdout().flush();
            // let mut row: String = row
            //     .iter()
            //     .enumerate()
            //     .map(|(col_num, c)| format!("{}", style(c.to_string()).red()))
            //     .collect();
            // row.push('\n');
            // println!("Printing line: \n {row}");

            writeln!(f)?;
        }
        Ok(())
    }
}

impl<const N: usize> From<&Map<N>> for MapDisplay {
    fn from(value: &Map<N>) -> Self {

        let dim = N;
        let mut display = MapDisplay::new(dim, 9, 5);

        let discovered_style = Style::new().on_black().on_bright().white();
        let visited_style = Style::new().on_white().black();

        for x in 0..dim {
            let pos = Position { x: x as u32, y: 0 };
            let dir = Direction::NegY;
            // let upper_wall = self.wall(&pos, &dir).expect("Wall should exist");
            // match upper_wall {
            // WallDiscoveryStatus::Undiscovered => {
            //     continue;
            // }
            // WallDiscoveryStatus::Exists(exists) => {
            let mut wall = display.wall_mut(pos, dir).expect("Wall should exist");
            //     if *exists {
            wall.full().set_char('+');
            wall.inner().set_char('-');
            wall.full().apply_style(discovered_style.clone().red());
            //             } else {
            //                 wall.inner().apply_style(discovered_style.clone());
            //             }
            //         }
            //     }
        }
        for y in 0..dim {
            let pos = Position { x: 0, y: y as u32 };
            let dir = Direction::NegX;
            // let upper_wall = self.wall(&pos, &dir).expect("Wall should exist");
            // match upper_wall {
            //     WallDiscoveryStatus::Undiscovered => {
            //         continue;
            //     }
            //     WallDiscoveryStatus::Exists(exists) => {
            let mut wall = display.wall_mut(pos, dir).expect("Wall should exist");
            // if *exists {
            wall.full().set_char('+');
            wall.inner().set_char('|');
            wall.full().apply_style(discovered_style.clone().red());
            //         } else {
            //             wall.inner().apply_style(discovered_style.clone());
            //         }
            //     }
            // }
        }

        for x in 0..dim {
            for y in 0..dim {
                let pos = Position {
                    x: x as u32,
                    y: y as u32,
                };
                let dir_right = Direction::PosX;
                let dir_down = Direction::PosY;

                let right_wall = value.wall(&pos, &dir_right).expect("Wall should exist");
                match right_wall {
                    WallDiscoveryStatus::Undiscovered => {}
                    WallDiscoveryStatus::Visited => {
                        let mut wall = display.wall_mut(pos, dir_right).expect("Wall should exist");
                        wall.inner().apply_style(visited_style.clone());
                    }
                    WallDiscoveryStatus::Exists(exists) => {
                        let mut wall = display.wall_mut(pos, dir_right).expect("Wall should exist");
                        if *exists {
                            wall.full().set_char('+');
                            wall.inner().set_char('|');
                            // wall.full().apply_style(Style::new().on_red());
                            wall.full().apply_style(discovered_style.clone().red());
                        } else {
                            // wall.inner().apply_style(Style::new().on_red());
                            wall.inner().apply_style(discovered_style.clone());
                        }
                    }
                }
                let lower_wall = value.wall(&pos, &dir_down).expect("Wall should exist");
                match lower_wall {
                    WallDiscoveryStatus::Undiscovered => {}
                    WallDiscoveryStatus::Visited => {
                        let mut wall = display.wall_mut(pos, dir_down).expect("Wall should exist");
                        wall.inner().apply_style(visited_style.clone());
                    }
                    WallDiscoveryStatus::Exists(exists) => {
                        let mut wall = display.wall_mut(pos, dir_down).expect("Wall should exist");
                        if *exists {
                            wall.full().set_char('+');
                            wall.inner().set_char('-');
                            // wall.full().apply_style(Style::new().on_blue());
                            wall.full().apply_style(discovered_style.clone().red());
                        } else {
                            // wall.inner().apply_style(Style::new().on_blue());
                            wall.inner().apply_style(discovered_style.clone());
                        }
                    }
                }
                let cell = value.cell(&pos).expect("Cell should exist");
                let mut cell_vis = display.cell_mut(pos).expect("Cell should exist");

                match cell {
                    CellDiscoveryStatus::Undiscovered => {
                        // cell_vis.apply_style(Style::new().on_yellow());
                    }
                    CellDiscoveryStatus::Discovered => {
                        cell_vis.apply_style(discovered_style.clone());
                    }
                    CellDiscoveryStatus::Visited => {
                        cell_vis.apply_style(visited_style.clone());
                    }
                }
            }
        }
        display
    }
}


