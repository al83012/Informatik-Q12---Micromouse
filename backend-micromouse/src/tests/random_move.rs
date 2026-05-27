use std::ops::DerefMut;

use rand::Rng;
use tracing::{debug, info};

use crate::{
    comm::micromouse_message::{
        Command, InterruptAction, InterruptStep, MeasurementInterrupt, MovementType,
        TransformedMovement,
    },
    transform::direction::RelativeDirection,
    utils::logging::run_test,
    map::map::WallDiscoveryStatus,
    tests::{self, test_map, test_world, TEST_MAP_SIZE},
    map::world_data::{CommandExecution, CommandStepResult, EndState, PartialWorldData, WorldData},
};

#[test]
pub fn do_random_moves() {
    let mut world = test_world(0.6);
    let mut partial_world = PartialWorldData::<TEST_MAP_SIZE>::default();
    run_test("info", || {
        info!(target: "tests/map/gen", "Test World:\n{world}");

        let commands = [
            Command {
                // Go forwards until you encounter a blockade or an opening on the right
                ty: MovementType::Move(TEST_MAP_SIZE as u8),
                interrupts: vec![
                    MeasurementInterrupt {
                        direction: RelativeDirection::Forward,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfBlocked,
                    },
                    MeasurementInterrupt {
                        direction: RelativeDirection::Right,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfOpen,
                    },
                ],
            },
            Command {
                ty: MovementType::Turn(-1),
                interrupts: vec![
                    MeasurementInterrupt {
                        direction: RelativeDirection::Right,
                        at_step: InterruptStep::At(0),
                        action: InterruptAction::StopIfBlocked
                    }
                ],
            },
            Command {
                ty: MovementType::Turn(2),
                interrupts: vec![
                    MeasurementInterrupt {
                        direction: RelativeDirection::Forward,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfOpen
                    }
                ]
            },
            // Escape from turning point
            Command {
                ty: MovementType::Move(1),
                interrupts: vec![]
            }
        ];

        let mut cmd_iter = commands.into_iter().cycle();
        let mut transf_move;
        for i in 0..500 {

            let next_cmd = cmd_iter.next().clone().expect("Iterator is cycling");

            debug!(target: "tests/map", "NEW TRANSF MOVE");
            let cmd_ty = next_cmd.ty;
            transf_move = TransformedMovement::new(cmd_ty, partial_world.mouse);

            debug!(target: "tests/map", "NEXT CMD: {next_cmd:?}");

            let mut command_executor = CommandExecution::new(world, next_cmd);
            let mut last_step = 0;

            loop {
                let res = command_executor.next();
                let is_continuing = res.is_continuing;
                let steps_done = res.num_of_finished_steps;
                let measurements = res.measurements;
                debug!(target: "tests/map", "STEP RESULTS: steps_done = {steps_done}, measurements = {measurements:?}");
                match is_continuing {
                    EndState::Ongoing(e) => {
                        debug!(target: "tests/map", "ONGOING --> apply move {:?}", partial_world.mouse);
                        if last_step + 1 == steps_done {
                            // crossed boundary?
                            
                            if let MovementType::Move(_) = cmd_ty {
                                let direction = partial_world.mouse.dir;
                                let from_cell = partial_world.mouse.pos;
                                if let Some(boundary) = partial_world.map.wall_mut(&from_cell, &direction) {
                                    *boundary = WallDiscoveryStatus::Visited;
                                }
                            }
                        }
                        last_step = steps_done;

                        //TODO: Auslagern
                        partial_world.mouse = transf_move
                            .at_step(steps_done)
                            .expect("Overexceeded move steps");
                        debug!(target: "tests/map", "     TO --> apply move {:?}", partial_world.mouse);
                        command_executor = e;
                        for m in measurements {
                            let discoveries = partial_world.apply_measurement(&m);
                            debug!(target: "tests/map", "DISCOVERY: {discoveries:?}");
                        }
                        info!(target: "tests/map", "CURRENT partial: \n{partial_world}");
                    }

                    EndState::Finished(w) => {
                        info!(target: "tests/map", "CMD FINISHED");
                        world = w;
                        for m in measurements {
                            let discoveries = partial_world.apply_measurement(&m);
                            debug!(target: "tests/map", "DISCOVERY: {discoveries:?}");
                        }
                        break;
                    }
                }
            }
        }
    });
}

#[test]
fn test_gen() {
    run_test("debug", || {
        let rand_map = test_map(0.3);
        info!(target: "tests/map/gen", "Finished building map: \n{rand_map}");
    })
}
