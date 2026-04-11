use std::ops::Deref;

use tracing::{error, info};

use crate::{
    comm::{
        micromouse_manager::MicromouseManager,
        micromouse_message::{
            Command, InterruptAction, InterruptStep, MeasurementInterrupt, MovementType,
        },
    },
    tests::TEST_MAP_SIZE,
    transform::direction::RelativeDirection,
    utils::logging::run_test,
};

#[ignore]
#[test]
fn test_conn_and_always_right() {
    run_test("trace", || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let micromouse_manager = MicromouseManager::<TEST_MAP_SIZE>::new()
                .await
                .expect("MICROMOUSE CONN ERR");

            let always_right_commands: [Command; _] = [
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
                    interrupts: vec![MeasurementInterrupt {
                        direction: RelativeDirection::Right,
                        at_step: InterruptStep::At(0),
                        action: InterruptAction::StopIfBlocked,
                    }],
                },
                Command {
                    ty: MovementType::Turn(2),
                    interrupts: vec![MeasurementInterrupt {
                        direction: RelativeDirection::Forward,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfOpen,
                    }],
                },
                // Escape from turning point
                Command {
                    ty: MovementType::Move(1),
                    interrupts: vec![],
                },
            ];

            let mut next_cmd_id = 0;

            loop {
                info!(target: "test/comm", "TEST TICK");
                tokio::select! {
                    _ = micromouse_manager.notified_empty_queue() => {
                        info!(target: "comm/mng", "EMPTY QUEUE");
                        let next_cmd = always_right_commands[next_cmd_id].clone();
                        info!(target: "comm/mng/cmd", "SENT NEXT COMMAND: {next_cmd:?}");
                        next_cmd_id = (next_cmd_id + 1) % always_right_commands.len();
                        micromouse_manager.send_command(next_cmd).await.expect("SENDING FAILED");
                        // Need next command
                    }
                    events = micromouse_manager.next() => {
                        match events {
                            Ok(events) => {
                                for event in events.deref().iter() {
                                    info!(target: "comm/mng/event", "RECEIVED EVENT: {event:?}");
                                }
                            } 
                            Err(e) => {
                                error!(target: "comm/mng/event", "ERROR: {e:?}")
                            }
                        }
                    }
                }
            }
        })
    });
}
