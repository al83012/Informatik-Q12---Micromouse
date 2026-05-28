use std::time::Duration;

use thiserror::Error;
use tokio::sync::mpsc::*;
use tracing::{error, info, instrument, Instrument};

use crate::{
    comm::{
        micromouse_manager::{
            self, MicromouseEvent, MicromouseManager, MicromouseManagerError, MicromouseMode,
        },
        micromouse_message::{Command, MicromouseMessage},
        website::{FrontendConnectionConfig, FrontendManager, FrontendMessage, FrontendResponse},
        websocket::{WsChannelConfig, WsChannelConnError, WsChannelConnInfo},
    },
    map::world_data::WorldData,
    strategy::{
        dyn_strategy_tree::{DynStrategyConfig, DynStrategyTree, DynStrategyTreeManager},
        strategies::follow_wall::{FollowWall, FollowWallConfig, WallDirection},
        strategy::{GoalPosition, Strategy},
        strategy_tree::{StrategyTree, StrategyTreeError},
    },
    transform::position::Position,
    utils::hyperlink_logging::process_span,
};

pub struct Process<const N: usize> {
    micromouse_manager: MicromouseManager<N>,
    frontend_manager: FrontendManager<N>,
    strategy_tree_manager: DynStrategyTreeManager<N>,
}

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Connection Error with Frontend")]
    FrontendConnError(WsChannelConnError),

    #[error("Connection Error with Micromouse")]
    MicromouseConnError(WsChannelConnError),

    #[error("Initial state not valid")]
    StrategyTreeError(#[from] StrategyTreeError),
}

impl ProcessError {
    pub fn frontend_conn(from: WsChannelConnError) -> Self {
        ProcessError::FrontendConnError(from)
    }
    pub fn micromouse_conn(from: WsChannelConnError) -> Self {
        ProcessError::MicromouseConnError(from)
    }
}

impl<const N: usize> Process<N> {
    pub const MICROMOUSE_PORT: u16 = 9001;
    pub const FRONTEND_PORT: u16 = 8090;
    #[instrument(name = "new Process", fields(description = "Create the main process"))]
    pub async fn new() -> Result<Self, ProcessError> {
        let frontend_manager = FrontendManager::new(
            Self::FRONTEND_PORT,
            FrontendConnectionConfig {
                batching_duration: Duration::from_millis(50),
                ws_channel_config: WsChannelConfig::default(),
            },
        )
        .await
        .map_err(ProcessError::frontend_conn)?;

        let micromouse_manager = MicromouseManager::new(Self::MICROMOUSE_PORT)
            .await
            .map_err(ProcessError::micromouse_conn)?;

        let strategy_tree_manager = DynStrategyTreeManager::new(
            WorldData::default().only_pos(),
            DynStrategyConfig::FollowWall(FollowWallConfig {
                follow_wall: WallDirection::Right,
            }),
            GoalPosition(Position {
                x: N as u32 - 1,
                y: N as u32 - 1,
            }),
            3,
            200,
        )?;

        Ok(Self {
            frontend_manager,
            micromouse_manager,
            strategy_tree_manager,
        })
    }

    #[instrument(
        skip_all,
        name = "run",
        fields(description = "Execute the main process")
    )]
    pub async fn run(mut self) {
        loop {
            let micromouse_response = self.micromouse_manager.await_next_read();
            let micromouse_conn_event = self.micromouse_manager.await_next_conn_event();
            let frontend_response = self.frontend_manager.next_read();
            let sendable_cmd = self.strategy_tree_manager.await_cmd();

            tokio::select! {
                cmd = sendable_cmd => {
                    info!(target: "proc", "New sendable cmd ({cmd:?})");
                    self.send_micromouse_cmd(cmd).await;
                }
                micromouse_conn_event = micromouse_conn_event => {
                    self.handle_micromouse_conn_event(micromouse_conn_event).await;
                }
                micromouse_msg = micromouse_response => {
                    let micromouse_event = self.micromouse_manager.process_next_read(micromouse_msg).await;
                    match micromouse_event {
                        Ok(events) => {
                            for event in events {
                                self.handle_micromouse_event(event).await;
                            }
                        }
                        Err(e) => {
                            self.handle_micromouse_error(e).await
                        }
                    }
                }
                frontend_msg = frontend_response => {
                    self.handle_frontend_command(frontend_msg).await;
                }
            }
        }
    }

    #[instrument(
        skip(self),
        name = "handle_micromouse_conn_event",
        fields(description = "Handles conn events by sending them to the frontend")
    )]
    pub async fn handle_micromouse_conn_event(
        &mut self,
        micromouse_conn_event: Option<WsChannelConnInfo>,
    ) {
        let Some(micromouse_conn_event) = micromouse_conn_event else {
            return;
        };
        self.frontend_manager
            .send(FrontendMessage::MicromouseConnectionEvent(
                micromouse_conn_event,
            ))
            .await;
    }

    #[instrument(
        skip(self),
        name = "handle_micromouse_error",
        fields(description = "Handle micromouse error and maybe propagate it")
    )]
    pub async fn handle_micromouse_error(&mut self, micromouse_error: MicromouseManagerError) {
        error!(target: "proc/err", "Micromouse Error: {micromouse_error:?}");
        self.frontend_manager
            .send(FrontendMessage::MicromouseEvent(MicromouseEvent::Error(
                micromouse_error,
            )))
            .await;
    }

    #[instrument(
        skip(self),
        name = "send_micromouse_cmd",
        fields(description = "Send command into command queue")
    )]
    pub async fn send_micromouse_cmd(&mut self, cmd: Command) {
        let cmd_id = self.micromouse_manager.send_command(cmd.clone()).await;

        self.frontend_manager
            .send(FrontendMessage::Debug(format!(
                "SENT COMMAND {cmd_id} {cmd:?}"
            )))
            .await;
        info!(target: "proc", "SENT COMMAND ({cmd:?}) with id {cmd_id}");
    }

    #[instrument(
        skip(self),
        name = "handle_micromouse_event",
        fields(
            description = "Handle micromouse event, apply it to the strategy tree and propagate it"
        )
    )]
    pub async fn handle_micromouse_event(&mut self, micromouse_event: MicromouseEvent) {
        match micromouse_event {
            MicromouseEvent::UpdatedMap(_) => {
                let current_map = self.micromouse_manager.current_world_lock().await.map;
                match self.strategy_tree_manager.update_filter(current_map) {
                    Ok(new_commands) => {
                        for command in new_commands {
                            let _send_res = self.micromouse_manager.send_command(command).await;
                        }
                    }
                    Err(e) => {
                        self.frontend_manager
                            .send(FrontendMessage::StrategyTreeError(e))
                            .await;
                    }
                }
            }
            MicromouseEvent::Restart => {
                if let Err(e) = self.strategy_tree_manager.set_pos_to_start_and_restart() {
                    self.frontend_manager
                        .send(FrontendMessage::StrategyTreeError(e))
                        .await;
                }
            }
            MicromouseEvent::Stop => {}
            MicromouseEvent::Continue => {}
            MicromouseEvent::Desync => {}
            MicromouseEvent::Error(ref e) => {
                self.handle_micromouse_error(e.clone()).await;
                return;
            }
            MicromouseEvent::DebugMessage(ref s) => {
                self.frontend_manager
                    .send(FrontendMessage::Debug(s.clone()))
                    .await
            }
            MicromouseEvent::RejectedOutcomes(_) => {}
            MicromouseEvent::FinishedCommand { .. } => {
                match self.strategy_tree_manager.finish_current_cmd() {
                    Ok(Some(end)) => {
                        self.frontend_manager
                            .send(FrontendMessage::StrategyEnd(end))
                            .await
                    }
                    Ok(None) => {}
                    Err(e) => {
                        self.frontend_manager
                            .send(FrontendMessage::StrategyTreeError(e.into()))
                            .await
                    }
                }
            }
            MicromouseEvent::UpdatePosition(p) => {} // MicromouseEvent::
        }

        self.frontend_manager
            .send(FrontendMessage::MicromouseEvent(micromouse_event))
            .await;
    }

    #[instrument(
        skip(self),
        name = "handle_frontend_command",
        fields(description = "Handle a new command / strategy change etc. sent from the frontend")
    )]
    pub async fn handle_frontend_command(&mut self, frontend_command: FrontendResponse<N>) {
        match frontend_command {
            FrontendResponse::StrategyChange(strategy_change_command) => {
                let Err(e) = self.strategy_tree_manager.modify(strategy_change_command) else {
                    return;
                };

                self.frontend_manager
                    .send(FrontendMessage::StrategyTreeError(e))
                    .await;
            }
            FrontendResponse::Pause => {
                self.micromouse_manager
                    .set_mode(MicromouseMode::Stopped)
                    .await;
            }
            FrontendResponse::Continue => {
                self.micromouse_manager
                    .set_mode(MicromouseMode::Running)
                    .await;
            }
        }
    }
}
