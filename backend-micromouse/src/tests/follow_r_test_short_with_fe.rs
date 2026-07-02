use std::time::Duration;

use tokio::time::{self, Instant, Interval};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing::{Instrument, Span};

use crate::comm::website::{FrontendConnectionConfig, FrontendManager, FrontendMessage};
use crate::comm::websocket::WsChannelConfig;
use crate::utils::hyperlink_logging::process_span;
use crate::{
    comm::{
        micromouse_manager::{MicromouseEvent, MicromouseManager},
        micromouse_message::{
            Command, InterruptAction, InterruptStep, MeasurementInterrupt, MovementType,
        },
    },
    tests::{
        micromouse_simulator::{self, MicromouseSimulator},
        TEST_MAP_SIZE,
    },
    transform::direction::RelativeDirection,
    utils::{
        hyperlink_logging::{enter_process, init_tree_logger},
    },
};

#[ignore]
#[test]
pub fn follow_r() {
    let world = super::test_map(0.5);
    init_tree_logger();
    let _s = enter_process("test");
    info!(target: "tests/map", "TEST WORLD:\n{world}");
    let mut micromouse_simulator = MicromouseSimulator::new(world, Duration::ZERO);
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut cancel = CancellationToken::new();
    let local_cancel = cancel.clone();

    rt.spawn(
        async move {
            tokio::select! {
                _ = cancel.cancelled() => {},
                _ = micromouse_simulator.run(2) => {},
            }
        }
        .instrument(process_span("simulator")),
    );
    info!(target: "comm/msg_log", "****************************************************************************************");
    rt.block_on(async {
        let micromouse_manager = MicromouseManager::<TEST_MAP_SIZE>::new(9001)
            .await
            .expect("MICROMOUSE CONN ERR");

        let mut frontend_manager = FrontendManager::<TEST_MAP_SIZE>::new(8090, FrontendConnectionConfig{
            batching_duration: Duration::from_millis(50),
            ws_channel_config: WsChannelConfig::default(),
        }).await.expect("FRONTEND CONN ERR");


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
                _ = micromouse_manager.await_space_in_queue() => {
                    let _s = enter_process("notify_empty_micromouse_queue");
                    info!(target: "comm/mng", "EMPTY QUEUE");
                    let next_cmd = always_right_commands[next_cmd_id].clone();
                    info!(target: "comm/mng/cmd", "SENT NEXT COMMAND: {next_cmd:?}");
                    next_cmd_id = (next_cmd_id + 1) % always_right_commands.len();
                    micromouse_manager.send_command(next_cmd).await;
                    // Need next command
                }
                msg = micromouse_manager.await_next_read() => {
                    let _s = enter_process("read_micromouse_msg");
                    // WARN: Have to move the code for parsing etc. here: It has to be blocking (at
                    // least relative to sending commands, as the order might get screwed up
                    // otherwise when the command sending takes precedence and cancels next() -->
                    // cannot be the condition for the select-clause
                    let events = micromouse_manager.process_next_read(msg).await;
                    match events {
                        Ok(events) => {
                            
                            for event in events.into_iter() {
                                info!(target: "comm/mng/event", "RECEIVED EVENT:\n{event:#?}");
                                if let MicromouseEvent::UpdatedMap(_) = event {
                                    info!(target: "comm/mng/event", "MAP UPDATE:\n{}", micromouse_manager.current_world_lock().await);
                                } else if let MicromouseEvent::UpdatePosition(_) = event {
                                    info!(target: "comm/mng/event", "POS UPDATE:\n{}", micromouse_manager.current_world_lock().await);
                                }
                                frontend_manager.send(FrontendMessage::MicromouseEvent(event)).await;

                            }
                        }
                        Err(e) => {
                            frontend_manager.send(FrontendMessage::MicromouseEvent(MicromouseEvent::Error(e.clone()))).await;
                            panic!("ERROR: {e:?}")
                        }
                    }
                }
                msg = frontend_manager.next_read() => {
                    let _s = process_span("read_frontend_msg");
                    info!(target: "comm/webs", "Frontend Response: {:?}", msg);
                }
                _ = recheck_cmd_tick.tick() => {
                    let _s = enter_process("recheck_tick_empty_micromouse_queue");
                    // Periodic updates so that we don't have to just rely on the chain of updates
                    // to continue
                    info!(target: "comm/mng/cmd", "RECHECKING QUEUE COUNT");
                    micromouse_manager.update_queue_count().await;
                    info!(target: "comm/mng/cmd", "FINISHED RECHECKING");
                }
            }
        }
    }.instrument(process_span("manager")));
    local_cancel.cancel();
    // });
}
