use crate::{
    transform::direction::Direction,
    map::map::{Map, MapInconsistencyError, WallDiscoveryStatus},
    map::measurement::{Measurement, MeasurementValue},
    transform::position::Position,
};

type TestMap = Map<4>;

#[test]
fn detect_wall_one_cell_ahead() {
    let mut map = TestMap::new();

    let m = Measurement {
        position: Position { x: 0, y: 0 },
        direction: Direction::PosX,
        value: MeasurementValue::Value { cells: 1 },
    };

    map.apply_measurement(&m).unwrap();

    assert_eq!(
        map.wall(&Position { x: 0, y: 0 }, &Direction::PosX),
        Some(&WallDiscoveryStatus::Exists(false))
    );

    assert_eq!(
        map.wall(&Position { x: 1, y: 0 }, &Direction::PosX),
        Some(&WallDiscoveryStatus::Exists(true))
    );
}

#[test]
fn outside_range_marks_no_walls() {
    let mut map = TestMap::new();

    let m = Measurement {
        position: Position { x: 0, y: 0 },
        direction: Direction::PosX,
        value: MeasurementValue::OutsideRange { at_least_cells: 3 },
    };

    map.apply_measurement(&m).unwrap();

    for i in 0..3 {
        assert_eq!(
            map.wall(&Position { x: i, y: 0 }, &Direction::PosX),
            Some(&WallDiscoveryStatus::Exists(false))
        );
    }
}

#[test]
fn multiple_empty_then_wall() {
    let mut map = TestMap::new();

    let m = Measurement {
        position: Position { x: 0, y: 0 },
        direction: Direction::PosY,
        value: MeasurementValue::Value { cells: 2 },
    };

    map.apply_measurement(&m).unwrap();

    assert_eq!(
        map.wall(&Position { x: 0, y: 0 }, &Direction::PosY),
        Some(&WallDiscoveryStatus::Exists(false))
    );

    assert_eq!(
        map.wall(&Position { x: 0, y: 1 }, &Direction::PosY),
        Some(&WallDiscoveryStatus::Exists(false))
    );

    assert_eq!(
        map.wall(&Position { x: 0, y: 2 }, &Direction::PosY),
        Some(&WallDiscoveryStatus::Exists(true))
    );
}

#[test]
fn conflicting_measurements_detected() {
    let mut map = TestMap::new();

    let m1 = Measurement {
        position: Position { x: 0, y: 0 },
        direction: Direction::PosX,
        value: MeasurementValue::Value { cells: 1 },
    };

    map.apply_measurement(&m1).unwrap();

    let m2 = Measurement {
        position: Position { x: 0, y: 0 },
        direction: Direction::PosX,
        value: MeasurementValue::OutsideRange { at_least_cells: 2 },
    };

    let result = map.apply_measurement(&m2);

    assert!(matches!(
        result,
        Err(MapInconsistencyError::Conflicting(_, _))
    ));
}

#[test]
fn repeated_same_measurement_ok() {
    let mut map = TestMap::new();

    let m = Measurement {
        position: Position { x: 0, y: 0 },
        direction: Direction::PosX,
        value: MeasurementValue::Value { cells: 1 },
    };

    map.apply_measurement(&m).unwrap();
    map.apply_measurement(&m).unwrap();
}

#[test]
fn outside_bounds_error() {
    let mut map = TestMap::new();

    let m = Measurement {
        position: Position { x: 3, y: 0 },
        direction: Direction::PosX,
        value: MeasurementValue::OutsideRange { at_least_cells: 5 },
    };

    let result = map.apply_measurement(&m);

    assert!(matches!(
        result,
        Err(MapInconsistencyError::OutsideBounds { .. })
    ));
}

#[test]
fn negative_direction_update() {
    let mut map = TestMap::new();

    let m = Measurement {
        position: Position { x: 2, y: 0 },
        direction: Direction::NegX,
        value: MeasurementValue::Value { cells: 1 },
    };

    map.apply_measurement(&m).unwrap();

    assert_eq!(
        map.wall(&Position { x: 2, y: 0 }, &Direction::NegX),
        Some(&WallDiscoveryStatus::Exists(false))
    );

    assert_eq!(
        map.wall(&Position { x: 1, y: 0 }, &Direction::NegX),
        Some(&WallDiscoveryStatus::Exists(true))
    );
}
