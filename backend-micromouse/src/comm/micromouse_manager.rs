use std::{
    collections::{HashMap, HashSet},
    sync::atomic::AtomicU32,
};

use tokio::sync::{Mutex, MutexGuard, Notify, RwLock, RwLockReadGuard};
use tracing::{debug, error, info, warn};

use crate::{
    comm::{
        micromouse_message::{
            Command, CommandId, CommandMessage, FormatError, MeasurementMessage,
            MicromouseResponse, StepNum,
        },
        website::DiscoveryMessage,
        websocket::{WsChannel, WsChannelConfig, WsChannelConnError},
    },
    map::{
        command_world_state::{
            CannotReachStep, CertainStepError, FilterMeasurementUpgradeError, FilterUpdate,
            FilteredCommandApplication, RejectedOutcomes,
        },
        map::MapInconsistencyError,
        world_data::WorldData,
    },
    transform::position::MouseTransform,
    utils::nonempty::{NonEmpty, PotentiallyNonEmpty},
};

pub struct MicromouseManager<const N: usize> {
    channel: WsChannel,
    next_cmd_id: AtomicU32,
    unconfirmed_cmd: Mutex<HashMap<CommandId, CommandMessage>>,
    mode: Mutex<MicromouseMode>,
    current_command: Mutex<Option<(FilteredCommandApplication<N>, CommandId)>>,
    current_world: RwLock<WorldData<N>>,
    notify_empty_queue: Notify,
    battery: Mutex<f32>,
}

pub struct InternalMapUpdate {
    new_transf: Option<MouseTransform>,
    discoveries: Option<NonEmpty<DiscoveryMessage>>,
    rejected_outcomes: Option<NonEmpty<RejectedOutcomes>>,
}

impl<const N: usize> MicromouseManager<N> {
    pub async fn new() -> Result<Self, WsChannelConnError> {
        info!(target: "comm/mng", "CREATING NEW MicromouseManager");
        let new_channel = WsChannel::new(WsChannelConfig::default(), 9001).await?;
        Ok(Self {
            channel: new_channel,
            next_cmd_id: AtomicU32::new(0),
            unconfirmed_cmd: Mutex::new(HashMap::new()),
            mode: Mutex::new(MicromouseMode::Stopped),
            current_command: Mutex::new(None),
            current_world: RwLock::new(WorldData::default()),
            battery: Mutex::new(100.0),
            notify_empty_queue: Notify::new(),
        })
    }

    pub async fn send_command(&self, cmd: Command) -> Result<CommandId, CommandSendError> {
        debug!(target: "comm/mng/cmd", "Adding cmd to queue {cmd:?}");
        let cmd_id = CommandId(
            self.next_cmd_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );

        debug!(target: "comm/mng/cmd", "Got Id {cmd_id:?}");
        let msg = CommandMessage { cmd, cmd_id };
        self.channel.send((&msg).into()).await;
        self.unconfirmed_cmd.lock().await.insert(cmd_id, msg);
        Ok(cmd_id)
    }

    // Returns boolean --> true = was resent, false = already finished, exited queue
    async fn resend(&self, cmd_id: CommandId) -> bool {
        warn!(target: "comm/mng/cmd", "RESENDING {cmd_id:?}");
        if let Some(msg) = self.unconfirmed_cmd.lock().await.get(&cmd_id) {
            self.channel.send(msg.into()).await;
            true
        } else {
            error!(target: "comm/mng/cmd", "CMD {cmd_id:?} does not exist");
            false
        }
    }

    /// WARN: Even when the event cannot be handled: Polling the next-function is necessary for the
    /// communication to continue (even though the channel will spin up a separate thread to keep
    /// the connection alive, it won't handle desyncs and the like)
    pub async fn next(&self) -> Result<NonEmpty<Vec<MicromouseEvent<N>>>, MicromouseManagerError> {
        info!(target: "comm/mng", "Read tick");
        loop {
            let next_response = &self.channel.read().await;
            if next_response.is_none() {
                return Err(MicromouseManagerError::ConnectionClosedPermanently);
            }
            let next_response: MicromouseResponse =
                next_response.as_ref().unwrap().to_string().try_into()?;
            match next_response {
                MicromouseResponse::Debug(msg) => {
                    debug!(target: "comm/mng/dbg", "READ DEBUG {msg}");
                    return Ok(NonEmpty::<_>::one(MicromouseEvent::DebugMessage(msg)));
                }
                MicromouseResponse::Measurement(measurement_message) => {
                    debug!(target: "comm/mng/measure", "READ MEASUREMENT {measurement_message:?}");
                    // Check whether command is new
                    self.update_current_command_id(measurement_message.from_cmd)
                        .await?;
                    let map_update = self
                        .update_cmd_application(
                            measurement_message.interrupt.at_step,
                            Some(measurement_message),
                        )
                        .await?;
                    if let Some(map_update_events) = map_update.into() {
                        debug!(target: "comm/mng/map", "Updated Map");
                        return Ok(map_update_events);
                    }
                }
                MicromouseResponse::CommandFinished(command_finished_message) => {
                    debug!(target: "comm/mng/cmd", "FINISHED COMMAND {command_finished_message:?}");
                    let just_finished_cmd = self.current_command.lock().await;
                    if just_finished_cmd.is_none() {
                        error!(target: "comm/mng/cmd", "NO CURRENT COMMAND TO FINISH");
                        return Err(MicromouseManagerError::CmdNotKnown(
                            command_finished_message.cmd_id,
                        ));
                    }
                    let (just_finished_cmd, just_finished_cmd_id) =
                        just_finished_cmd.as_ref().expect("Already checked");
                    let step_num = match command_finished_message.reason {
                        Some(i) => i.occurence.at_step,
                        None => {
                            // The step at which it ended must have been the max step that way
                            // available (since the max step is per definition the step at which an
                            // interrupt or max_step is triggered)
                            just_finished_cmd.max_step() as u32
                        }
                    };
                    debug!(target: "comm/mng/cmd", "FINISHED AT STEP {step_num}");
                    // do a last position-update (in case we didn't get a measurement in the last
                    // step and have to update it to its last transf that way)
                    self.update_cmd_application(step_num, None).await?;
                    self.clear_current_command().await;
                    let require_new = self.unconfirmed_cmd.lock().await.is_empty();
                    if require_new {
                        debug!(target: "comm/mng/cmd", "REQUIRE NEW");
                        self.notify_empty_queue.notify_waiters();
                    }
                    return Ok(NonEmpty::<_>::one(MicromouseEvent::FinishedCommand {
                        cmd_id: *just_finished_cmd_id,
                        // If we are not aware of a command in the queue, we will have to get a new
                        // one
                        require_new,
                    }));
                }
                MicromouseResponse::Desync(command_ids) => {
                    warn!(target: "comm/mng/cmd", "DESYNC {command_ids:?}");
                    for c in command_ids {
                        if !self.resend(c).await {
                            error!(target: "comm/mng/cmd", "RESEND FAILED");
                            return Err(MicromouseManagerError::CmdConfirmThenReqested(c));
                        }
                    }
                }
                MicromouseResponse::Stop => {
                    info!(target: "comm/mng", "STOPPED");
                    *self.mode.lock().await = MicromouseMode::Stopped;
                    return Ok(NonEmpty::<_>::one(MicromouseEvent::Stop));
                }
                MicromouseResponse::Restart => {
                    info!(target: "comm/mng", "RESTARTED");
                    *self.mode.lock().await = MicromouseMode::Running;
                    self.restart().await;
                    return Ok(NonEmpty::<_>::one(MicromouseEvent::Restart));
                }
                MicromouseResponse::Continue => {
                    info!(target: "comm/mng", "CONTINUED");
                    *self.mode.lock().await = MicromouseMode::Running;
                    return Ok(NonEmpty::<_>::one(MicromouseEvent::Continue));
                }
                MicromouseResponse::Battery(b_100) => {
                    debug!(target: "comm/mng/battery", "BATTERY: {b_100}/100");
                    let f_b = b_100 as f32 / 100.0;
                    *self.battery.lock().await = f_b;
                }
            }
        }
    }
    pub async fn restart(&self) {
        debug!(target: "comm/mng", "Doing Restart...");
        self.next_cmd_id
            .store(0, std::sync::atomic::Ordering::SeqCst);
        *self.mode.lock().await = MicromouseMode::Running;
        *self.current_command.lock().await = None;
        *self.current_world.write().await = WorldData::default();
        *self.unconfirmed_cmd.lock().await = HashMap::new();
        self.notify_empty_queue.notify_waiters();
        debug!(target: "comm/mng", "RESTART COMPLETE");
    }

    /// Uses a measurement or a command finished message (-> measurment = None) to update the
    /// current position (as those messages carry a step-number)
    async fn update_cmd_application(
        &self,
        step_number: StepNum,
        measurement: Option<MeasurementMessage>,
    ) -> Result<InternalMapUpdate, MicromouseManagerError> {
        debug!(target: "comm/mng/map", "UPDATE INTERNAL MAP (step = {step_number}, measurement = {measurement:?})");
        let mut current_cmd = self.current_command.lock().await;

        let (current_cmd_application, id) = current_cmd
            .as_mut()
            .ok_or(MicromouseManagerError::MeasurementWithoutAssociatedCmd)?;
        if step_number > current_cmd_application.step_with_termination() as u32 {
            error!(target: "comm/mng/map", "SHOULD ALREADY HAVE TERMINATED IN PREVIOUS STEP");
            return Err(MicromouseManagerError::CmdTooLong(*id));
        }
        let world_at_step = current_cmd_application
            .at_start_certain_step(step_number)
            .map_err(MicromouseManagerError::from)?;

        let new_transf = {
            if world_at_step.mouse != self.current_world.read().await.mouse {
                Some(world_at_step.mouse)
            } else {
                None
            }
        };

        let filter_update = if let Some(m) = measurement {
            // First get the transform to transform the measurement (first reaches the cell, then does
            // the measurement)
            let new_transf = world_at_step.mouse;
            let transf_measurement = m.transform_by(&new_transf);

            let filter_update =
                current_cmd_application.apply_measurement_to_filter(transf_measurement)?;
            let new_map = current_cmd_application
                .at_start_certain_step(step_number)
                .map_err(MicromouseManagerError::from)?;
            *self.current_world.write().await = new_map.clone().into();
            filter_update
        } else {
            // since there is no new measurement, we can just take the world_at_step as the new
            // world
            *self.current_world.write().await = world_at_step.clone().into();
            // RejectedOutcomes::empty()
            FilterUpdate {
                discoveries: None,
                rejections: None,
            }
        };

        let internal_map_update = InternalMapUpdate {
            rejected_outcomes: filter_update.rejections,
            discoveries: filter_update.discoveries,
            new_transf,
        };

        Ok(internal_map_update)
    }

    pub async fn notified_empty_queue(&self) {
        self.notify_empty_queue.notified().await
    }

    /// Sets the current_cmd to none, returns the old one; Should NEVER return None
    async fn clear_current_command(&self) -> Option<FilteredCommandApplication<N>> {
        let mut current_cmd = self.current_command.lock().await;
        let old_cmd = current_cmd.take();
        old_cmd.map(|x| x.0)
    }

    /// Checks whether the cmd_id contained in the response is a new one --> Would mean, that the
    /// previous command **has** to be finished and a new cmd started
    async fn update_current_command_id(
        &self,
        response_cmd_id: CommandId,
    ) -> Result<(), MicromouseManagerError> {
        // let (transformed_cmd, mut current_cmd) = self.current_command.lock().await;
        let mut current_cmd = self.current_command.lock().await;
        if current_cmd.is_none() {
            // Get the command we just started from the list of unconfirmed commands (commands that
            // were sent but have not yet sent any processing information) --> Should exist, error
            // otherwise
            let new_cmd = self
                .unconfirmed_cmd
                .lock()
                .await
                .remove(&response_cmd_id)
                .ok_or(MicromouseManagerError::CmdNotKnown(response_cmd_id))?;

            // Storing the starting state of the new command, so that we can easily calculate,
            // where the mouse currently is
            *current_cmd = Some((
                FilteredCommandApplication::new(
                    Some(self.current_world.read().await.clone()),
                    new_cmd.cmd,
                ),
                response_cmd_id,
            ));
            return Ok(());
        }

        let current_cmd_id = current_cmd.as_ref().unwrap().1;

        if current_cmd_id == response_cmd_id {
            // No change; current command = new command
            Ok(())
        } else {
            // Started a new command without exiting the previous one
            error!(target: "comm/mng/cmd", "MISSING CMD FINISH, TRIED STARTING NEW ONE");
            Err(MicromouseManagerError::CmdStartBeforeFinish {
                new_cmd: response_cmd_id,
                unfinished_cmd: current_cmd_id,
            })
        }
    }

    pub async fn current_world_lock(&self) -> RwLockReadGuard<'_, WorldData<N>> {
        self.current_world.read().await
    }
}

#[derive(Debug, PartialEq)]
pub enum MicromouseMode {
    Stopped,
    Running,
}

#[derive(Debug)]
pub enum MicromouseEvent<const N: usize> {
    UpdatePosition(MouseTransform),
    UpdatedMap(NonEmpty<DiscoveryMessage>),
    FinishedCommand {
        cmd_id: CommandId,
        require_new: bool,
    },
    Stop,
    Restart,
    Continue,
    Error(MicromouseManagerError),
    DebugMessage(String),
    RejectedOutcomes(NonEmpty<RejectedOutcomes>),
}

#[derive(Debug)]
pub enum CommandSendError {
    /// The strategy was manually stopped, no command should be sent, it will be voided
    StoppedExecution,
}

#[derive(Debug)]
pub enum MicromouseManagerError {
    ConnectionClosedPermanently,
    UnknownResponse(FormatError<MicromouseResponse>),
    CmdConfirmThenReqested(CommandId),
    CmdStartBeforeFinish {
        new_cmd: CommandId,
        unfinished_cmd: CommandId,
    },
    CmdNotKnown(CommandId),
    MeasurementWithoutAssociatedCmd,
    CmdTooLong(CommandId),
    ImpossiblePosition(CommandId),
    CannotReachStep(CannotReachStep),
    MapInconsistency(FilterMeasurementUpgradeError),
    PathNotProven(CertainStepError),
}

impl From<FormatError<MicromouseResponse>> for MicromouseManagerError {
    fn from(value: FormatError<MicromouseResponse>) -> Self {
        Self::UnknownResponse(value)
    }
}

impl From<CannotReachStep> for MicromouseManagerError {
    fn from(value: CannotReachStep) -> Self {
        Self::CannotReachStep(value)
    }
}

impl From<FilterMeasurementUpgradeError> for MicromouseManagerError {
    fn from(value: FilterMeasurementUpgradeError) -> Self {
        Self::MapInconsistency(value)
    }
}

impl From<CertainStepError> for MicromouseManagerError {
    fn from(value: CertainStepError) -> Self {
        // While updating the internal position to match the new one, it was discovered that the
        // measurements that were received do not prove, that this step was allowed
        // (We didn't prove that the micromouse would not have interrupted before)
        Self::PathNotProven(value)
    }
}

impl<const N: usize> From<InternalMapUpdate> for Option<NonEmpty<Vec<MicromouseEvent<N>>>> {
    fn from(value: InternalMapUpdate) -> Self {
        let mut vec = Vec::with_capacity(3);
        if let Some(x) = value.new_transf {
            vec.push(MicromouseEvent::UpdatePosition(x))
        }
        if let Some(x) = value.discoveries {
            vec.push(MicromouseEvent::UpdatedMap(x))
        }
        if let Some(x) = value.rejected_outcomes {
            vec.push(MicromouseEvent::RejectedOutcomes(x))
        }
        vec.non_empty()
    }
}
