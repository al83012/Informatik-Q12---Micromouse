use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tungstenite::{Message, Utf8Bytes};

use crate::{
    comm::micromouse_message::{
        CommandMessage, InterruptAction, InterruptOccurence, MeasurementMessage,
        MeasurementOccurrence, TransformedMovement,
    },
    map::{
        map::Map,
        measurement::{self, MeasurementValue},
        world_data::{self, WorldData},
    },
    transform::position::MouseTransform,
};

pub struct MicromouseSimulator<const N: usize> {
    full_map: WorldData<N>,
}

impl<const N: usize> MicromouseSimulator<N> {
    pub fn new(full_map: Map<N>) -> Self {
        Self {
            full_map: WorldData {
                map: full_map,
                mouse: MouseTransform::default(),
            },
        }
    }
    pub async fn run(&mut self, max_measure_depth: u8) {
        let (mut ws_stream, response) = tokio_tungstenite::connect_async("ws://localhost:9001")
            .await
            .expect("Connection failed");
        info!(target: "test/sim", " < Connection Response = {response:?}");

        'next_cmd: while let Some(msg) = ws_stream.next().await {
            let msg = match msg {
                Ok(o) => o,
                Err(e) => {
                    error!(target: "test/sim", "Connection Error = {e}");
                    continue;
                }
            };

            let Message::Text(msg) = msg else {
                error!(target: "test/sim", "Got non-text message: {msg}");
                continue;
            };
            let msg = CommandMessage::try_from(msg.to_string());
            if let Err(e) = &msg {
                error!(target: "test/sim", "Got non-cmd message: {e}");
                continue;
            }
            let msg = msg.expect("Checked");

            let current_cmd = msg.cmd;
            let transformed_move = TransformedMovement::new(current_cmd.ty, self.full_map.mouse);

            info!(target: "test/sim", "AT CMD SIM {}", msg.cmd_id);

            for i in 0..=current_cmd.max_step_count() {
                // tokio::time::sleep(Duration::from_millis(500)).await;
                info!(target: "test/sim", "Sim at step {i}");
                let current_transf = transformed_move.at_step(i).expect("In valid range");
                self.full_map.mouse = current_transf;
                for interrupt in current_cmd.interrupts.iter() {
                    if !interrupt.at_step.matches(i) {
                        continue;
                    }
                    let measurement = self
                        .full_map
                        .measure(interrupt.direction, max_measure_depth);
                    let (depth, is_sensorlimit) = match measurement.value {
                        MeasurementValue::OutsideRange { at_least_cells } => (at_least_cells, true),
                        MeasurementValue::Value { cells } => (cells, false),
                    };
                    let measurement_msg = MeasurementMessage {
                        from_cmd: msg.cmd_id,
                        interrupt: MeasurementOccurrence {
                            direction: interrupt.direction,
                            at_step: i as u32,
                        },
                        depth,
                        is_sensorlimit,
                    };
                    info!(target: "test/sim", "Measured: {measurement_msg:?}");

                    ws_stream
                        .send(Message::Text(Utf8Bytes::from(String::from(
                            measurement_msg,
                        ))))
                        .await
                        .expect("Panic on sending measurement");

                    if (interrupt.action == InterruptAction::StopIfBlocked && depth == 0)
                        || (interrupt.action == InterruptAction::StopIfOpen && depth != 0)
                    {
                        ws_stream
                            .send(Message::Text(Utf8Bytes::from(format!(
                                "CMD-FINISHED {}",
                                msg.cmd_id
                            ))))
                            .await
                            .expect("Panic on sendinc cmd_finished");
                        continue 'next_cmd;
                    }
                }
            }
            self.full_map.mouse = transformed_move
                .at_step(current_cmd.max_step_count())
                .expect("in bounds");
            ws_stream
                .send(Message::Text(Utf8Bytes::from(format!(
                    "CMD-FINISHED {}",
                    msg.cmd_id
                ))))
                .await
                .expect("Panic on sendinc cmd_finished");
            continue 'next_cmd;
        }

        // Self { full_map }
    }
}
