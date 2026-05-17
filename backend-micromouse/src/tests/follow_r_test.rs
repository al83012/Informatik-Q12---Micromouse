use std::{ops::Deref, time::Duration};

use tokio::time::{self, Instant, Interval};
use tracing::{error, info, instrument};

use crate::{
    comm::{
        micromouse_manager::{MicromouseEvent, MicromouseManager},
        micromouse_message::{
            Command, InterruptAction, InterruptStep, MeasurementInterrupt, MovementType,
        },
    },
    tests::TEST_MAP_SIZE,
    transform::direction::RelativeDirection,
    utils::{
        hyperlink_logging::init_tree_logger,
        logging::{init_logging, run_test},
    },
};

#[ignore]
#[test]
#[instrument(name = "follow_r_test_short")]
pub fn follow_r_and_conn() {
    // let guards = init_logging();
    init_tree_logger();
    info!(target: "comm/msg_log", "****************************************************************************************");
    // run_test("trace", || {
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

        let mut recheck_cmd_tick = time::interval(Duration::from_millis(500));

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
                msg = micromouse_manager.await_next_read() => {
                    // WARN: Have to move the code for parsing etc. here: It has to be blocking (at
                    // least relative to sending commands, as the order might get screwed up
                    // otherwise when the command sending takes precedence and cancels next() -->
                    // cannot be the condition for the select-clause
                    let events = micromouse_manager.process_next_read(msg).await;
                    match events {
                        Ok(events) => {
                            for event in events.deref().iter() {
                                info!(target: "comm/mng/event", "RECEIVED EVENT:\n{event:#?}");
                                if let MicromouseEvent::UpdatedMap(_) = event {
                                    info!(target: "comm/mng/event", "MAP UPDATE:\n{}", micromouse_manager.current_world_lock().await);
                                } else if let MicromouseEvent::UpdatePosition(_) = event {
                                    info!(target: "comm/mng/event", "POS UPDATE:\n{}", micromouse_manager.current_world_lock().await);
                                }
                            }
                        }
                        Err(e) => {
                            panic!("ERROR: {e:?}");
                        }
                    }
                }
                _ = recheck_cmd_tick.tick() => {
                    // Periodic updates so that we don't have to just rely on the chain of updates
                    // to continue
                    info!(target: "comm/mng/cmd", "RECHECKING QUEUE COUNT");
                    micromouse_manager.update_queue_count().await;
                    info!(target: "comm/mng/cmd", "FINISHED RECHECKING");
                }
            }
        }
    })
    // });
}
