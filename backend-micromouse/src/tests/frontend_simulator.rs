use std::{
    collections::{hash_map, HashMap},
    time::Duration,
};

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
        website::{BatchedFrontendMessage, FrontendMessage, FrontendResponse},
    }, map::{
        map::Map,
        measurement::{self, MeasurementValue},
        world_data::{self, WorldData},
    }, strategy::{
        dyn_strategy_tree::{DynStrategyConfig, StrategyChangeCommand},
        strategies::{
            dbg_known_path::DbgKnownPathConfig,
            depth_first::DepthFirstConfig,
            flood_fill::{FFCost, FloodFillConfig},
            follow_wall::{FollowWallConfig, WallDirection},
            utils::depth_first_base::PathRanking,
        },
        strategy::GoalPosition,
        strategy_tree::AbsolutePathId,
        visuals::{PathSegment, TreeVisualEvent},
    }, transform::position::{MouseTransform, Position}, utils::{frontend_display::FrontendDisplay, hyperlink_logging::{LinkFileName, enter_process, process_span}},
};

const SIZE: usize = super::TEST_MAP_SIZE;

pub struct FrontendSimulator {
    current_strat_id: usize,
    routine_strategy_change_interval: Duration,
    display: FrontendDisplay<SIZE>,
}

impl FrontendSimulator {
    #[instrument(name = "new FrontendSimulator")]
    pub fn new(strategy_change_interval: Duration) -> Self {
        Self {
            current_strat_id: 1,
            routine_strategy_change_interval: strategy_change_interval,
            display: FrontendDisplay::new(),
        }
    }
    #[instrument(skip(self), name = "run")]
    pub async fn run(&mut self) {
        let (mut ws_stream, response) = tokio_tungstenite::connect_async("ws://localhost:8090")
            .await
            .expect("Connection failed");
        info!(target: "test/sim", " < Connection Response = {response:?}");

        tokio::time::sleep(Duration::from_secs(2)).await;
        // let initial_msg = FrontendResponse::StrategyChange(StrategyChangeCommand {
        //     set_goal: None,
        //     reset_map: true,
        //     set_strategy: Some(DynStrategyConfig::<10>::DepthFirst(DepthFirstConfig {
        //         path_ranking: PathRanking::TowardsGoal,
        //     })),
        // });

        let mut tick = tokio::time::interval(self.routine_strategy_change_interval);

        loop {
            let Some(frontend_msg_batch) = (tokio::select! {
                msg = ws_stream.next() => msg,
                _ = tick.tick() => {
                    warn!(target: "tests/sim/webs", "ROUTINE STRATEGY CHANGE");

                        let next_strat = self.other_strategy();
                        // panic!("NEXT STRAT: {next_strat:?}");
                        ws_stream
                            .send(Message::Text(Utf8Bytes::from(next_strat)))
                            .await
                            .expect("Sending should not just panic");
                    continue;
                }

            }) else {
                error!(target: "tests/sim/webs", "Received channel close");
                break;
            };
            let Ok(frontend_msg_batch) = frontend_msg_batch else {
                error!(target: "tests/sim/webs", "Error while receiving from backend {:#?}", frontend_msg_batch.expect_err("Checked"));
                ws_stream
                    .close(Some(tungstenite::protocol::CloseFrame {
                        code: CloseCode::Error,
                        reason: Utf8Bytes::from_static("Encountered Error on read from backend"),
                    }))
                    .await;
                return;
            };
            let Ok(frontend_msg_batch) = frontend_msg_batch.to_text() else {
                // Ping or other msg
                continue;
            };
            let Ok(frontend_msg_batch) =
                serde_json::de::from_str::<BatchedFrontendMessage>(frontend_msg_batch)
            else {
                warn!(target: "tests/sim/webs", "Non-parseable msg {frontend_msg_batch:#?}");
                continue;
            };


            self.display.update(&frontend_msg_batch);
            info!(target: "test/sim/webs/display", "Frontend Display: \n{}", self.display);


            info!(target: "test/sim/webs", "RECEIVED FRONTEND MSG {frontend_msg_batch:#?}");

            for frontend_msg in frontend_msg_batch.0 {
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
                        // panic!("NEXT STRAT: {next_strat:?}");
                        ws_stream
                            .send(Message::Text(Utf8Bytes::from(next_strat)))
                            .await
                            .expect("Sending should not just panic");
                    }
                    FrontendMessage::MicromouseConnectionEvent(ws_channel_conn_info) => {
                        info!(target: "tests/sim/webs", "Micromouse Connection Event >> {ws_channel_conn_info:?}");
                    }
                    FrontendMessage::Debug(msg) => {
                        info!(target: "tests/sim/webs", "DEBUG {msg}");
                    }
                    FrontendMessage::ConfirmLastChange => {
                        info!(target: "tests/sim/webs", "Confirmed Last Change");
                    }
                    FrontendMessage::VisualEvent(visual_event) => {
                        info!(target: "tests/sim/webs", "VISUAL EVENT {visual_event:?}")
                    }
                }
            }
        }
    }

    fn other_strategy(&mut self) -> String {
        let current_strat_id = self.current_strat_id % 2;

        let next: &DynStrategyConfig<10> = &[
            // DynStrategyConfig::DepthFirst(DepthFirstConfig {
            //     path_ranking: PathRanking::Undefined,
            //     prune_dead_ends: false,
            // }),
            // DynStrategyConfig::FollowWall(FollowWallConfig {
            //     follow_wall: WallDirection::Right,
            //     measure_all: true,
            // }),
            // DynStrategyConfig::DepthFirst(DepthFirstConfig {
            //     forward_first: true,
            // }),
            DynStrategyConfig::DbgKnownPath(DbgKnownPathConfig {
                move_cost: FFCost {
                    base_value: 10,
                    streak_reduction_factor: 0.5,
                },
                rotation_cost: FFCost {
                    base_value: 15,
                    streak_reduction_factor: 0.0,
                },
            }),
            // DynStrategyConfig::FloodFill(FloodFillConfig {
            //     move_cost: FFCost {
            //         base_value: 10,
            //         streak_reduction_factor: 1.0,
            //     },
            //     rotation_cost: FFCost {
            //         base_value: 15,
            //         streak_reduction_factor: 1.0,
            //     },
            //     exploration_incentive: 7,
            // }),
            DynStrategyConfig::FloodFill(FloodFillConfig {
                move_cost: FFCost {
                    base_value: 10,
                    streak_reduction_factor: 0.5,
                },
                rotation_cost: FFCost {
                    base_value: 15,
                    streak_reduction_factor: 0.0,
                },
                exploration_incentive: 0,
            }), // DynStrategyConfig::DepthFirst(DepthFirstConfig {
                //     path_ranking: PathRanking::TowardsGoal,
                //     prune_dead_ends: true,
                // }),
                // DynStrategyConfig::FollowWall(FollowWallConfig {
                //     follow_wall: WallDirection::Right,
                //     measure_all: false,
                // }),
        ][current_strat_id];

        let next_pos = [Position { x: 0, y: 0 }, Position { x: 8, y: 8 }][current_strat_id];

        let strat_change = StrategyChangeCommand {
            set_strategy: Some(next.clone()),
            reset_map: !self.current_strat_id.is_multiple_of(2),
            set_goal: Some(GoalPosition(next_pos)),
        };

        self.current_strat_id = current_strat_id + 1;
        serde_json::ser::to_string_pretty(&FrontendResponse::StrategyChange(strat_change))
            .expect("Should be pareseable")
    }
}
