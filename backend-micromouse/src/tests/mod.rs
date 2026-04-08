use std::collections::{HashMap, HashSet};

use rand::Rng;
use tracing::debug;

use crate::{
    direction::{Direction, DirectionNormalizedVector},
    map::{CellDiscoveryStatus, Map, WallDiscoveryStatus},
    position::{MouseTransform, Position, PositionOffset},
    world_data::{SIM_MAX_DEPTH, WorldData},
};

mod random_move;
mod test_map_discoveries;
mod visual_tests;

pub const TEST_MAP_SIZE: usize = 12;

pub fn test_world(loopiness: f32) -> WorldData<TEST_MAP_SIZE> {
    let map = test_map(loopiness);
    let mouse = MouseTransform::default();
    WorldData { map, mouse }
}

pub fn test_map(loopiness: f32) -> Map<TEST_MAP_SIZE> {
    let loopiness = loopiness.clamp(0.0, 1.0);
    let mut map = Map::new();

    // connection from    low_group --> higher_groups
    // [row][col] = [y][x]
    let mut groups = Vec::<Vec<usize>>::new();
    let mut possible_connections = HashMap::<Position, HashSet<Direction>>::new();

    debug!(target: "tests/map/gen", "Initializing...");

    for y in 0..TEST_MAP_SIZE {
        let mut row_group = Vec::with_capacity(TEST_MAP_SIZE);
        for x in 0..TEST_MAP_SIZE {
            let pos = Position {
                x: x as u32,
                y: y as u32,
            };
            *map.wall_mut(&pos, &Direction::PosX).expect("should exist") =
                WallDiscoveryStatus::Exists(true);
            *map.wall_mut(&pos, &Direction::PosY).expect("should exist") =
                WallDiscoveryStatus::Exists(true);
            *map.cell_mut(&pos).expect("should exist") = CellDiscoveryStatus::Discovered;
            row_group.push(x + y * TEST_MAP_SIZE);
            let mut connection_partners = HashSet::new();
            if x + 1 < TEST_MAP_SIZE {
                connection_partners.insert(Direction::PosX);
            }
            if y + 1 < TEST_MAP_SIZE {
                connection_partners.insert(Direction::PosY);
            }
            if !connection_partners.is_empty() {
                possible_connections.insert(pos, connection_partners);
            }
        }
        groups.push(row_group);
    }

    debug!(target: "tests/map/gen", "Initialized: All blocked...\n{map}");

    let mut rand = rand::thread_rng();
    while !possible_connections.is_empty() {
        let num_of_conn_sources = possible_connections.len();
        debug!(target: "tests/map/gen", "Sources left = {num_of_conn_sources}");
        let rand_key = {
            let mut keys = possible_connections.keys();
            keys.nth(rand.gen_range(0..num_of_conn_sources))
                .expect("should exist")
                .clone()
        };
        debug!(target: "tests/map/gen", "Source = {rand_key}");
        let conn_dir = {
            let conn_options = possible_connections
                .get_mut(&rand_key)
                .expect("should exist");
            debug!(target: "tests/map/gen", "Conn options = {conn_options:?}");
            let conn_dir = {
                conn_options
                    .iter()
                    .nth(rand.gen_range(0..conn_options.len()))
                    .expect("should exist")
                    .clone()
            };
            debug!(target: "tests/map/gen", "Conn dir = {conn_dir}");
            conn_options.remove(&conn_dir);
            debug!(target: "tests/map/gen", "Source conns left = {conn_options:?}");
            if conn_options.is_empty() {
                debug!(target: "tests/map/gen", "Source empty --> removing");
                possible_connections.remove(&rand_key);
            }
            conn_dir
        };

        let wall = map.wall_mut(&rand_key, &conn_dir).expect("should exist");
        *wall = crate::map::WallDiscoveryStatus::Exists(false);

        let source_group = groups[rand_key.y as usize][rand_key.x as usize];
        debug!(target: "tests/map/gen", "Source group = {source_group}");

        let other_pos = (rand_key
            + PositionOffset::from(DirectionNormalizedVector::from(conn_dir)))
        .expect("should be within bounds");
        debug!(target: "tests/map/gen", "Other pos = {other_pos}");
        let other_group = groups[other_pos.y as usize][other_pos.x as usize];

        debug!(target: "tests/map/gen", "Other group = {other_group}");

        for x in 0..TEST_MAP_SIZE {
            for y in 0..TEST_MAP_SIZE {
                let found_group = groups[y][x];
                let pos = Position {
                    x: x as u32,
                    y: y as u32,
                };
                // replace all from source_group to other group
                if found_group == other_group {
                    debug!(target: "tests/map/gen", "{pos} {found_group}->{source_group}");
                    groups[y][x] = source_group;
                }
            }
        }

        for x in 0..TEST_MAP_SIZE {
            for y in 0..TEST_MAP_SIZE {
                if groups[y][x] == source_group {
                    let pos = Position {
                        x: x as u32,
                        y: y as u32,
                    };
                    debug!(target: "tests/map/gen", "test loop @ {pos}");
                    // is in the new bigger group; remove loops
                    if let Some(possible_conn_partners) = possible_connections.get_mut(&pos) {
                        for x in possible_conn_partners.clone() {
                            debug!(target: "tests/map/gen", "  --> in dir {x}");
                            if let Some(other_pos) =
                                (pos + PositionOffset::from(DirectionNormalizedVector::from(x)))
                            {
                                debug!(target: "tests/map/gen", "  = at pos {other_pos}");
                                if groups[other_pos.y as usize][other_pos.x as usize]
                                    == source_group
                                {
                                    debug!(target: "tests/map/gen", "remove loop: {pos} & {other_pos}");
                                    //Loop within a group --> Not a viable connection partner anymore
                                    //If loopiness is high: actually doesn't delete all the loops
                                    //immediately
                                    if rand::random::<f32>() >= loopiness {
                                        possible_conn_partners.remove(&x);
                                    }
                                }
                            }
                        }
                        if possible_conn_partners.is_empty() {
                            possible_connections.remove(&pos);
                        }
                    }
                }
            }
        }

        debug!(target: "tests/map/gen","\n{}", map);
    }

    map
}
