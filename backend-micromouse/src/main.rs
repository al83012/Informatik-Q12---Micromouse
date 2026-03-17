use crate::{direction::Direction, map::Map, measurement::{Measurement, MeasurementValue}, position::Position};

pub mod map;
pub mod measurement;
pub mod direction;
pub mod position;
pub mod comm;

#[cfg(test)]
pub mod tests;

fn main() {



    // let m = Measurement {
    //     position: Position { x: 0, y: 0 },
    //     direction: Direction::PosY,
    //     value: MeasurementValue::Value { cells: 2 },
    // };
    //
    // let mut map = Map::<4>::new();
    // map.update_discovery(&m).unwrap();
    //

    // println!("{}", map);

}
