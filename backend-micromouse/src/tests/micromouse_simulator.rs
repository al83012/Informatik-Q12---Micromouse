use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, span, warn, Instrument, Level};
use tungstenite::{Message, Utf8Bytes};

use crate::{
    comm::micromouse_message::{
        CommandMessage, InterruptAction, InterruptOccurence, MeasurementMessage,
        MeasurementOccurrence, MicromouseResponse, TransformedMovement,
    },
    map::{
        map::Map,
        measurement::{self, MeasurementValue},
        world_data::{self, WorldData},
    },
    transform::position::MouseTransform,
    utils::hyperlink_logging::{enter_process, process_span, LinkFileName},
};

pub struct MicromouseSimulator<const N: usize> {
    full_map: WorldData<N>,
    is_restarting: bool,
}

impl<const N: usize> MicromouseSimulator<N> {
    pub fn new(full_map: Map<N>) -> Self {
        Self {
            full_map: WorldData {
                map: full_map,
                mouse: MouseTransform::default(),
            },
            is_restarting: false,
        }
    }
    #[instrument(skip(self), name = "run")]
    pub async fn run(&mut self, max_measure_depth: u8) {
        let (mut ws_stream, response) = tokio_tungstenite::connect_async("ws://localhost:9001")
            .await
            .expect("Connection failed");
        info!(target: "test/sim", " < Connection Response = {response:?}");

        ws_stream
            .send(Message::Text(Utf8Bytes::from("RESTART".to_string())))
            .await
            .expect("Error sending opening msg");
        self.is_restarting = true;
        ws_stream
            .send(Message::Text(Utf8Bytes::from("CONTINUE".to_string())))
            .await
            .expect("Error sending opening msg");

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
            if self.is_restarting {
                if msg == "RESTART-CONFIRM" {
                    self.is_restarting = false;
                }
                continue;
            }
            let msg = CommandMessage::try_from(msg.to_string());
            if let Err(e) = &msg {
                warn!(target: "test/sim", "Got non-cmd message: {e}");
                continue;
            }
            let msg = msg.expect("Checked");

            let continue_next_cmd: bool = async {
                let current_cmd = msg.cmd;
                let transformed_move =
                    TransformedMovement::new(current_cmd.ty, self.full_map.mouse);

                info!(target: "test/sim", "AT CMD SIM {}", msg.cmd_id);

                for i in 0..=current_cmd.max_step_count() {
                    // tokio::time::sleep(Duration::from_millis(50)).await;
                    let continue_next_cmd: bool = async {
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
                                MeasurementValue::OutsideRange { at_least_cells } => {
                                    (at_least_cells, true)
                                }
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
                                // continue 'next_cmd;
                                return true;
                            }
                        }
                        false
                    }
                    .instrument(process_span(format!("sim_span_step_{i}")))
                    .await;
                    if continue_next_cmd {
                        return true;
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
                true
            }
            .instrument(span!(
                Level::INFO,
                "process",
                name = format!("sim_{}", msg.cmd_id.link()),
                link_cmd_id = msg.cmd_id.link()
            ))
            .await;

            if continue_next_cmd {
                continue 'next_cmd;
            }
        }

    }
}
