use console::Style;
use tracing::info;

use crate::{
    comm::micromouse_message::MovementType,
    map::map::Map,
    transform::{
        direction::Direction,
        position::{MouseTransform, Position},
    },
    utils::{
        logging::init_logging,
        map_display::{self, MapDisplay, MapDisplayWrite},
        path::{Path, PathReference},
    },
};

#[test]
pub fn test_path_return() {
    init_logging();

    let mut path = Path::new(MouseTransform {
        pos: Position { x: 0, y: 0 },
        dir: Direction::PosX,
    });

    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 5, y: 0 },
        dir: Direction::PosX,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 5, y: 0 },
        dir: Direction::PosY,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 5, y: 2 },
        dir: Direction::PosY,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 5, y: 2 },
        dir: Direction::PosX,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 8, y: 2 },
        dir: Direction::PosX,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 8, y: 2 },
        dir: Direction::PosY,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 8, y: 5 },
        dir: Direction::PosY,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 8, y: 5 },
        dir: Direction::NegX,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 7, y: 5 },
        dir: Direction::NegX,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 7, y: 5 },
        dir: Direction::PosY,
    }));
    assert!(path.connect_to(MouseTransform {
        pos: Position { x: 7, y: 8 },
        dir: Direction::PosY,
    }));

    {
        let map = Map::<10>::new();
        let mut map_display = MapDisplay::from(&map);
        info!(target: "test/path", "{path:?}");

        let mut path_ref = PathReference::new(path.clone(), &mut map_display);

        path_ref.set_char('*');

        let map_str = format!("\n{map_display}");

        info!(target: "test/path", "{}", map_str);
    }

    assert_eq!(
        path.return_to(MouseTransform {
            pos: Position { x: 7, y: 8 },
            dir: Direction::NegX
        }),
        Ok(vec![MovementType::Turn(-1),])
    );

    {
        let map = Map::<10>::new();
        let mut map_display = MapDisplay::from(&map);
        let mut path_ref = PathReference::new(path.clone(), &mut map_display);

        path_ref.set_char('*');

        let map_str = format!("\n{map_display}");

        info!(target: "test/path", "{}", map_str);
    }
    assert_eq!(
        path.return_to(MouseTransform {
            pos: Position { x: 7, y: 7 },
            dir: Direction::NegX
        }),
        Ok(vec![
            MovementType::Turn(-1),
            MovementType::Move(1),
            MovementType::Turn(1)
        ])
    );

    {
        let map = Map::<10>::new();
        let mut map_display = MapDisplay::from(&map);
        let mut path_ref = PathReference::new(path.clone(), &mut map_display);

        path_ref.set_char('*');

        let map_str = format!("\n{map_display}");

        info!(target: "test/path", "{}", map_str);
    }

    assert_eq!(
        path.return_to(MouseTransform {
            pos: Position { x: 7, y: 2 },
            dir: Direction::NegY,
        }),
        Ok(vec![
            MovementType::Turn(-1),
            MovementType::Move(2),
            MovementType::Turn(-1),
            MovementType::Move(1),
            MovementType::Turn(1),
            MovementType::Move(3),
            MovementType::Turn(1),
            MovementType::Move(1),
            MovementType::Turn(-1)
        ])
    );
    {
        let map = Map::<10>::new();
        let mut map_display = MapDisplay::from(&map);
        let mut path_ref = PathReference::new(path.clone(), &mut map_display);

        path_ref.set_char('*');

        let map_str = format!("\n{map_display}");

        info!(target: "test/path", "{}", map_str);
    }

    assert_eq!(
        path.return_to(MouseTransform {
            pos: Position { x: 7, y: 2 },
            dir: Direction::PosX,
        }),
        Ok(vec![MovementType::Turn(-1),])
    );
    {
        let map = Map::<10>::new();
        let mut map_display = MapDisplay::from(&map);
        let mut path_ref = PathReference::new(path.clone(), &mut map_display);

        path_ref.set_char('*');

        let map_str = format!("\n{map_display}");

        info!(target: "test/path", "{}", map_str);
    }
    assert_eq!(
        path.return_to(MouseTransform {
            pos: Position { x: 7, y: 2 },
            dir: Direction::NegX,
        }),
        Ok(vec![MovementType::Turn(2),])
    );
    {
        let map = Map::<10>::new();
        let mut map_display = MapDisplay::from(&map);
        let mut path_ref = PathReference::new(path.clone(), &mut map_display);

        path_ref.set_char('*');

        let map_str = format!("\n{map_display}");

        info!(target: "test/path", "{}", map_str);
    }
    assert_eq!(
        path.return_to(MouseTransform {
            pos: Position { x: 5, y: 2 },
            dir: Direction::PosY,
        }),
        Ok(vec![MovementType::Move(2),MovementType::Turn(1)])
    );
    {
        let map = Map::<10>::new();
        let mut map_display = MapDisplay::from(&map);
        let mut path_ref = PathReference::new(path.clone(), &mut map_display);

        path_ref.set_char('*');

        let map_str = format!("\n{map_display}");

        info!(target: "test/path", "{}", map_str);
    }
}
