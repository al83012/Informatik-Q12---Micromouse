use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, span, warn, Instrument, Level};
use tungstenite::{protocol::frame::coding::CloseCode, Message, Utf8Bytes};

use crate::{
    comm::{
        micromouse_manager::MicromouseEvent,
        micromouse_message::{
            CommandMessage, InterruptAction, InterruptOccurence, MeasurementMessage,
            MeasurementOccurrence, MicromouseResponse, TransformedMovement,
        },
        website::{FrontendMessage, FrontendResponse},
    },
    map::{
        map::Map,
        measurement::{self, MeasurementValue},
        world_data::{self, WorldData},
    },
    strategy::{
        dyn_strategy_tree::{DynStrategyConfig, StrategyChangeCommand},
        strategies::{
            depth_first::DepthFirstConfig,
            follow_wall::{FollowWallConfig, WallDirection},
        },
        strategy::GoalPosition,
    },
    transform::position::{MouseTransform, Position},
    utils::hyperlink_logging::{enter_process, process_span, LinkFileName},
};

pub struct FrontendSimulator {
    current_strat_id: usize,
}

impl FrontendSimulator {
    #[instrument(name = "new FrontendSimulator")]
    pub fn new() -> Self {
        Self {
            current_strat_id: 0,
        }
    }
    #[instrument(skip(self), name = "run")]
    pub async fn run(&mut self) {
        let (mut ws_stream, response) = tokio_tungstenite::connect_async("ws://localhost:9002")
            .await
            .expect("Connection failed");
        info!(target: "test/sim", " < Connection Response = {response:?}");

        ws_stream
            .send(Message::Text(Utf8Bytes::from("RESTART".to_string())))
            .await
            .expect("Error sending opening msg");

        while let Some(frontend_msg) = ws_stream.next().await {
            let Ok(frontend_msg) = frontend_msg else {
                error!(target: "tests/sim/webs", "Error while receiving from backend {}", frontend_msg.expect_err("Checked"));
                ws_stream
                    .close(Some(tungstenite::protocol::CloseFrame {
                        code: CloseCode::Error,
                        reason: Utf8Bytes::from_static("Encountered Error on read from backend"),
                    }))
                    .await;
                return;
            };
            let Ok(frontend_msg) = frontend_msg.to_text() else {
                // Ping or other msg
                continue;
            };
            let frontend_msg: FrontendMessage = serde_json::de::from_str(frontend_msg)
                .expect("Message from backend should always be parseable");

            info!(target: "test/sim/webs", "RECEIVED FRONTEND MSG {frontend_msg:?}");

            match frontend_msg {
                FrontendMessage::MicromouseEvent(MicromouseEvent::Error(e)) => {
                    error!(target: "tests/sim/webs", "Error in micromouse: {e:?}");
                    ws_stream
                        .close(Some(tungstenite::protocol::CloseFrame {
                            code: CloseCode::Error,
                            reason: Utf8Bytes::from_static(
                                "Encountered Error on read from backend",
                            ),
                        }))
                        .await;
                    return;
                }
                FrontendMessage::MicromouseEvent(_) => {}
                FrontendMessage::StrategyTreeError(strategy_tree_error) => {
                    error!(target: "tests/sim/webs", "Error in strategy tree: {strategy_tree_error:?}");
                    ws_stream
                        .close(Some(tungstenite::protocol::CloseFrame {
                            code: CloseCode::Error,
                            reason: Utf8Bytes::from_static(
                                "Encountered Error on read from backend",
                            ),
                        }))
                        .await;
                    return;
                }
                FrontendMessage::StrategyEnd(strategy_end_state) => {
                    match strategy_end_state {
                        crate::strategy::strategy::StrategyEndState::NoPossibleAction(msg) => {
                            warn!(target: "tests/sim/webs", "Strategy cannot continue: {msg}");
                        }
                        crate::strategy::strategy::StrategyEndState::ReachedGoal => {
                            info!(target: "tests/sim/webs", "Strategy ended; Reached Goal")
                        }
                    }
                    let next_strat = self.other_strategy();
                    ws_stream
                        .send(Message::Text(Utf8Bytes::from(next_strat)))
                        .await
                        .expect("Sending should not just panic");
                }
                FrontendMessage::MicromouseConnectionEvent(ws_channel_conn_info) => todo!(),
                FrontendMessage::Debug(_) => todo!(),
                FrontendMessage::ConfirmLastChange => todo!(),
            }
        }
    }

    fn other_strategy(&mut self) -> String {
        let current_strat_id = self.current_strat_id % 2;

        let next: &DynStrategyConfig<10> = &[
            DynStrategyConfig::DepthFirst(DepthFirstConfig {
                forward_first: true,
            }),
            DynStrategyConfig::FollowWall(FollowWallConfig {
                follow_wall: WallDirection::Right,
            }),
        ][current_strat_id];

        let next_pos = [Position { x: 0, y: 0 }, Position { x: 5, y: 5 }][current_strat_id];

        self.current_strat_id = current_strat_id + 1;

        let strat_change = StrategyChangeCommand {
            set_postion: None,
            set_strategy: Some(next.clone()),
            reset_map: true,
            set_goal: Some(GoalPosition(next_pos)),
        };

        serde_json::ser::to_string_pretty(&FrontendResponse::StrategyChange(strat_change))
            .expect("Should be pareseable")
    }
}
