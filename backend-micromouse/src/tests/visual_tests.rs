use std::io::Write;

use tracing::{debug, info};

use super::*;
use crate::direction::Direction;
use crate::logging::run_test;
use crate::map::Map;
use crate::measurement::{Measurement, MeasurementValue};
use crate::position::Position;

#[test]
fn large_discovery_visual_test() {
    run_test("info",|| {
        info!(target: "tests/map/display","TESTING MAP");
        let mut map: Map<8> = Map::new();

        let measurements = vec![
            // Start at (0,0), open corridor to the right
            Measurement {
                position: Position { x: 0, y: 0 },
                direction: Direction::PosX,
                value: MeasurementValue::Value { cells: 5 },
            },
            // From (5,0), go down
            Measurement {
                position: Position { x: 5, y: 0 },
                direction: Direction::PosY,
                value: MeasurementValue::Value { cells: 4 },
            },
            // From (5,4), go left
            Measurement {
                position: Position { x: 5, y: 4 },
                direction: Direction::NegX,
                value: MeasurementValue::Value { cells: 3 },
            },
            // From (2,4), small dead-end downward
            Measurement {
                position: Position { x: 2, y: 4 },
                direction: Direction::PosY,
                value: MeasurementValue::Value { cells: 1 },
            },
            // From (2,5), dead end confirmed (no further cells)
            Measurement {
                position: Position { x: 2, y: 5 },
                direction: Direction::PosY,
                value: MeasurementValue::Value { cells: 0 },
            },
            // Some partial exploration elsewhere (unknown continues)
            Measurement {
                position: Position { x: 1, y: 1 },
                direction: Direction::PosY,
                value: MeasurementValue::OutsideRange { at_least_cells: 3 },
            },
            Measurement {
                position: Position { x: 3, y: 2 },
                direction: Direction::PosX,
                value: MeasurementValue::OutsideRange { at_least_cells: 2 },
            },
        ];

        for m in measurements {
            debug!(target: "tests/map/display","Applying measurement {m:?}");
            map.apply_measurement(&m).unwrap();
        }
        // debug!(target: "tests/map/display", "PRINTING");
        // println!("PRINTING");
        let map_str = format!("{}", map);
        // std::io::stdout().flush().expect("Error flushing");
        debug!(target: "tests/map/display","\n{map}");
        // println!("{map}");

        // No asserts — purely for visual inspection
    });
}

#[test]
fn smaller_discovery_visual_test() {
    run_test("debug",|| {
        let mut map = Map::<8>::new();
        let measurements = [
            Measurement {
                position: Position { x: 0, y: 0 },
                direction: Direction::PosX,
                value: MeasurementValue::OutsideRange { at_least_cells: 3 },
            },
            Measurement {
                position: Position { x: 1, y: 0 },
                direction: Direction::PosX,
                value: MeasurementValue::OutsideRange { at_least_cells: 3 },
            },
            Measurement {
                position: Position { x: 2, y: 0 },
                direction: Direction::PosX,
                value: MeasurementValue::OutsideRange { at_least_cells: 3 },
            },
            Measurement {
                position: Position { x: 3, y: 0 },
                direction: Direction::PosX,
                value: MeasurementValue::Value { cells: 3 },
            },
            Measurement {
                position: Position { x: 3, y: 0 },
                direction: Direction::PosY,
                value: MeasurementValue::Value { cells: 2 },
            },
            Measurement {
                position: Position { x: 3, y: 1 },
                direction: Direction::PosY,
                value: MeasurementValue::Value { cells: 1 },
            },
            Measurement {
                position: Position { x: 3, y: 2 },
                direction: Direction::PosY,
                value: MeasurementValue::Value { cells: 0 },
            },
            Measurement {
                position: Position { x: 3, y: 2 },
                direction: Direction::NegX,
                value: MeasurementValue::Value { cells: 3 },
            },
            Measurement {
                position: Position { x: 2, y: 2 },
                direction: Direction::NegX,
                value: MeasurementValue::Value { cells: 2 },
            },
            Measurement {
                position: Position { x: 1, y: 2 },
                direction: Direction::NegX,
                value: MeasurementValue::Value { cells: 1 },
            },
            Measurement {
                position: Position { x: 1, y: 2 },
                direction: Direction::NegY,
                value: MeasurementValue::Value { cells: 1 },
            },
            Measurement {
                position: Position { x: 1, y: 1 },
                direction: Direction::NegY,
                value: MeasurementValue::Value { cells: 0 },
            },
            Measurement {
                position: Position { x: 1, y: 1 },
                direction: Direction::PosX,
                value: MeasurementValue::Value { cells: 2 },
            },
        ];

        info!(target: "tests/map/display", "MAP\n{map}");
        for m in measurements {
            map.apply_measurement(&m).expect("Error while measuring");
            info!(target: "tests/map/display", "MAP\n{map}");
        }
    });
}
