use tracing::info;

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

#[tokio::test]
#[ignore]
async fn test_conn_and_always_right() {
    run_test("debug", || {
        tokio::runtime::Handle::current().block_on(async {
            let micromouse_manager  = MicromouseManager::<TEST_MAP_SIZE>::new().await.expect("MICROMOUSE CONN ERR");

            let ALWAYS_RIGHT_COMMANDS: [Command; _] = [
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
                tokio::select! {
                    _ = micromouse_manager.notified_empty_queue() => {
                        let next_cmd = ALWAYS_RIGHT_COMMANDS[next_cmd_id].clone();
                        info!(target: "comm/mng/cmd", "SENT NEXT COMMAND: {next_cmd:?}");
                        next_cmd_id = (next_cmd_id + 1) % ALWAYS_RIGHT_COMMANDS.len();
                        micromouse_manager.send_command(next_cmd).await.expect("SENDING FAILED");
                        // Need next command
                    }
                    events = micromouse_manager.next() => {
                        for event in events {
                            info!(target: "comm/mng/event", "RECV EVENT: {event:?}");
                        }
                    }
                }
            }
        })
    });
}
