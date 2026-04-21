use std::{
    collections::HashMap,
    ops::Deref,
    sync::atomic::{AtomicBool, AtomicU32},
};

use console::Style;
use futures_util::future::pending;
use serde::{Deserialize, Serialize};
use tokio::sync::{watch, Mutex, MutexGuard, RwLock, RwLockReadGuard};
use tracing::{debug, error, info, warn};
use tungstenite::Message;

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
        world_data::WorldData,
    },
    transform::position::MouseTransform,
    utils::nonempty::{NonEmpty, PotentiallyNonEmpty},
};

pub struct MicromouseManager<const N: usize> {
    channel: WsChannel,
    next_cmd_send_id: AtomicU32,
    unconfirmed_cmd: Mutex<HashMap<CommandId, CommandMessage>>,
    next_cmd_process_id: AtomicU32,
    mode: Mutex<MicromouseMode>,
    current_command: Mutex<Option<(FilteredCommandApplication<N>, CommandId)>>,
    current_world: RwLock<WorldData<N>>,
    queue_length_sender: tokio::sync::watch::Sender<usize>,
    queue_length_receiver: Mutex<tokio::sync::watch::Receiver<usize>>,
    target_queue_length: usize,
    battery: Mutex<f32>,
    start_marker: AtomicBool,
}
#[derive(Debug)]
pub struct InternalMapUpdate {
    new_transf: Option<MouseTransform>,
    discoveries: Option<NonEmpty<DiscoveryMessage>>,
    rejected_outcomes: Option<NonEmpty<RejectedOutcomes>>,
}

impl<const N: usize> MicromouseManager<N> {
    pub async fn new() -> Result<Self, WsChannelConnError> {
        info!(target: "comm/mng", "CREATING NEW MicromouseManager");
        let new_channel = WsChannel::new(WsChannelConfig::default(), 9001).await?;
        let (queue_length_sender, queue_length_receiver) = watch::channel(0);
        queue_length_sender
            .send(0)
            .expect("Stays const, shouldn't panic");
        Ok(Self {
            channel: new_channel,
            next_cmd_send_id: AtomicU32::new(0),
            unconfirmed_cmd: Mutex::new(HashMap::new()),
            next_cmd_process_id: AtomicU32::new(0),
            mode: Mutex::new(MicromouseMode::Stopped),
            current_command: Mutex::new(None),
            current_world: RwLock::new(WorldData::default()),
            battery: Mutex::new(100.0),

            start_marker: AtomicBool::from(true),
            queue_length_sender,
            queue_length_receiver: Mutex::new(queue_length_receiver),
            target_queue_length: 3,
        })
    }

    pub async fn send_command(&self, cmd: Command) -> Result<CommandId, CommandSendError> {
        debug!(target: "comm/mng/cmd", "Adding cmd to queue {cmd:?}");
        let cmd_id = CommandId(
            self.next_cmd_send_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );

        debug!(target: "comm/mng/cmd", "Got Id {cmd_id:?}");
        let msg = CommandMessage { cmd, cmd_id };
        let cmd_msg = (&msg).into();
        let cmd_str: String = (&msg).into();

        info!(target: "comm/msg_log", "> {cmd_str:?}");
        self.channel.send(cmd_msg).await;
        let mut unconfirmed_cmd_queue = self.unconfirmed_cmd.lock().await;
        unconfirmed_cmd_queue.insert(cmd_id, msg);
        debug!(target: "comm/mng/cmd", "COMMAND QUEUE LEN = {}", unconfirmed_cmd_queue.len());
        self.queue_length_sender
            .send(unconfirmed_cmd_queue.len())
            .expect("Channels stay const");
        Ok(cmd_id)
    }

    // Returns boolean --> true = was resent, false = already finished, exited queue
    async fn resend(&self, cmd_id: CommandId) -> bool {
        warn!(target: "comm/mng/cmd", "RESENDING {cmd_id:?}");
        if let Some(msg) = self.unconfirmed_cmd.lock().await.get(&cmd_id) {
            info!(target: "comm/msg_log", "> {msg:?}");
            self.channel.send(msg.into()).await;
            true
        } else {
            error!(target: "comm/mng/cmd", "CMD {cmd_id:?} does not exist");
            false
        }
    }

    pub async fn next_read(&self) -> Option<Message> {
        self.channel.read().await
    }

    /// WARN: Even when the event cannot be handled: Polling the next-function is necessary for the
    /// communication to continue (even though the channel will spin up a separate thread to keep
    /// the connection alive, it won't handle desyncs and the like)
    pub async fn next(
        &self,
        read: Option<Message>,
    ) -> Result<Vec<MicromouseEvent>, MicromouseManagerError> {
        info!(target: "comm/mng", "Read tick");
        if read.is_none() {
            error!(target: "comm/mng", "CONNECTION CLOSED DELIBERATELY & PERMANENTLY");
            return Err(MicromouseManagerError::ConnectionClosedPermanently);
        }
        let next_response_str = read.as_ref().unwrap().to_string();
        info!(target: "comm/msg_log", "< {next_response_str}");
        let next_response: MicromouseResponse = next_response_str.try_into()?;
        info!(target: "comm/mng", "NEXT RESPONSE: {next_response:?}");
        match next_response {
            MicromouseResponse::Debug(msg) => {
                debug!(target: "comm/mng/dbg", "READ DEBUG {msg}");
                Ok(vec![MicromouseEvent::DebugMessage(msg)])
            }
            MicromouseResponse::Measurement(measurement_message) => {
                info!(target: "comm/mng/measure", "READ MEASUREMENT {measurement_message:?}");
                // Check whether command is new
                let mut current_cmd = self.current_command.lock().await;
                self.update_current_command_id(measurement_message.from_cmd, &mut current_cmd)
                    .await?;
                let map_update = self
                    .update_cmd_application(
                        measurement_message.interrupt.at_step,
                        Some(measurement_message),
                        &mut current_cmd,
                    )
                    .await?;
                debug!(target: "comm/mng/map", "Updated Map\n{:#?}", &map_update);
                Ok(map_update.into())
            }
            MicromouseResponse::CommandFinished(command_finished_message) => {
                debug!(target: "comm/mng/cmd", "FINISHED COMMAND {command_finished_message:?}");
                let mut just_finished_cmd = self.current_command.lock().await;
                self.update_current_command_id(
                    command_finished_message.cmd_id,
                    &mut just_finished_cmd,
                )
                .await?;
                if just_finished_cmd.is_none() {
                    error!(target: "comm/mng/cmd", "NO CURRENT COMMAND TO FINISH");
                    return Err(MicromouseManagerError::CmdNotKnown(
                        command_finished_message.cmd_id,
                    ));
                }
                let step_num = match command_finished_message.reason {
                    Some(i) => i.occurence.at_step,
                    None => {
                        // The step at which it ended must have been the max step that way
                        // available (since the max step is per definition the step at which an
                        // interrupt or max_step is triggered)
                        let max_step = just_finished_cmd
                            .as_ref()
                            .expect("checked")
                            .0
                            .step_with_termination() as u32;
                        debug!(target: "comm/mng/cmd", "No step_num given: max_step = {max_step}");
                        max_step
                    }
                };
                debug!(target: "comm/mng/cmd", "FINISHED AT STEP {step_num}");
                // do a last position-update (in case we didn't get a measurement in the last
                // step and have to update it to its last transf that way)
                debug!(target: "comm/mng/cmd", "Currently unconfirmed {:?}", self.unconfirmed_cmd.lock().await);
                let map_update = self
                    .update_cmd_application(step_num, None, &mut just_finished_cmd)
                    .await;
                if let Err(e) = map_update {
                    warn!(target: "comm/mng/cmd", "Err {e:?} while updating map; Still need to clear it");
                    self.clear_current_command(&mut just_finished_cmd).await;
                    return Err(e);
                }
                let map_update = map_update.expect("Checked");
                let finished_cmd_id = just_finished_cmd.as_ref().expect("checked").1;
                self.clear_current_command(&mut just_finished_cmd).await;
                let require_new = self.unconfirmed_cmd.lock().await.is_empty();
                let map_update: Vec<MicromouseEvent> = map_update.into();
                Ok(vec![MicromouseEvent::FinishedCommand {
                    cmd_id: finished_cmd_id,
                    // If we are not aware of a command in the queue, we will have to get a new
                    // one
                    require_new,
                }]
                .into_iter()
                .chain(map_update)
                .collect())
            }
            MicromouseResponse::Desync(command_ids) => {
                warn!(target: "comm/mng/cmd", "DESYNC {command_ids:?}");
                for c in command_ids {
                    if !self.resend(c).await {
                        error!(target: "comm/mng/cmd", "RESEND FAILED");
                        return Err(MicromouseManagerError::CmdConfirmThenReqested(c));
                    }
                }
                Ok(vec![])
            }
            MicromouseResponse::Stop => {
                info!(target: "comm/mng", "STOPPED");
                *self.mode.lock().await = MicromouseMode::Stopped;
                Ok(vec![MicromouseEvent::Stop])
            }
            MicromouseResponse::Restart => {
                info!(target: "comm/mng", "RESTARTED");
                self.restart().await;
                *self.mode.lock().await = MicromouseMode::Running;
                Ok(vec![MicromouseEvent::Restart])
            }
            MicromouseResponse::Continue => {
                info!(target: "comm/mng", "CONTINUED");
                *self.mode.lock().await = MicromouseMode::Running;
                Ok(vec![MicromouseEvent::Continue])
            }
            MicromouseResponse::Battery(b_100) => {
                debug!(target: "comm/mng/battery", "BATTERY: {b_100}/100");
                let f_b = b_100 as f32 / 100.0;
                *self.battery.lock().await = f_b;
                Ok(vec![])
            }
        }
    }
    pub async fn restart(&self) {
        debug!(target: "comm/mng", "Doing Restart...");
        self.next_cmd_send_id
            .store(0, std::sync::atomic::Ordering::SeqCst);
        *self.mode.lock().await = MicromouseMode::Running;
        *self.current_command.lock().await = None;
        *self.current_world.write().await = WorldData::default();
        *self.unconfirmed_cmd.lock().await = HashMap::new();
        self.start_marker
            .store(true, std::sync::atomic::Ordering::SeqCst);
        // self.notify_empty_queue.notify_waiters();
        debug!(target: "comm/mng", "RESTART COMPLETE");
    }

    async fn remove_unordered(
        &self,
        cmd_to_remove: CommandId,
    ) -> Result<(), MicromouseManagerError> {
        warn!(target: "comm/mng/cmd", "REMOVING COMMAND OUT OF ORDER {cmd_to_remove}");
        let mut unconfirmed = self.unconfirmed_cmd.lock().await;
        let removed = unconfirmed.remove(&cmd_to_remove);
        if removed.is_none() {
            error!(target: "comm/mng/cmd", "CMD already removed or never sent");
            Err(MicromouseManagerError::CmdNotKnown(cmd_to_remove))
        } else {
            warn!(target: "comm/mng/cmd", "Remove successful");
            Ok(())
        }
    }

    /// Uses a measurement or a command finished message (-> measurment = None) to update the
    /// current position (as those messages carry a step-number)
    async fn update_cmd_application<'a>(
        &self,
        step_number: StepNum,
        measurement: Option<MeasurementMessage>,
        current_cmd: &mut MutexGuard<'a, Option<(FilteredCommandApplication<N>, CommandId)>>,
    ) -> Result<InternalMapUpdate, MicromouseManagerError> {
        debug!(target: "comm/mng/map", "UPDATE INTERNAL MAP (step = {step_number}, measurement = {measurement:?})");
        // let mut  current_cmd = self.current_command.lock().await;

        let (current_cmd_application, id) = current_cmd
            .as_mut()
            .ok_or(MicromouseManagerError::MeasurementWithoutAssociatedCmd)?;
        if step_number > current_cmd_application.step_with_termination() as u32 {
            error!(target: "comm/mng/map", "SHOULD ALREADY HAVE TERMINATED IN PREVIOUS STEP");
            return Err(MicromouseManagerError::CmdTooLong(*id));
        }
        let world_at_step = current_cmd_application.at_start_certain_step(step_number);
        let world_at_step = match world_at_step {
            Ok(world_at_step) => {
                info!(target: "comm/mng/map", "AT START OF MEASUREMENT: \n{world_at_step}");
                world_at_step
            }
            Err(e) => {
                // WARN: LIFE MUST GO ON:
                error!(target: "comm/mng/map", "CERTAIN STEP ERROR: DID NOT PROVE THAT STEP {} OF {:?} WAS REACHABLE",
                    step_number,
                    current_cmd_application.command()
                );
                return Err(MicromouseManagerError::from(e));
            }
        };
        // .map_err(MicromouseManagerError::from)?;

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

            info!(target: "comm/mng/map", "WORLD AFTER MEASUREMENT: \n{new_map}");
            let mut current_world = self.current_world.write().await;
            *current_world = new_map.clone().into();

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

        if let Some(rej) = &filter_update.rejections {
            for rej in rej.deref().rejected_outcome_ids.iter() {
                let style = Style::new().strikethrough();
                debug!(target: "comm/mng/cmd", "REJECTED = {}", style.apply_to( format!("{rej:?}")));
            }
        }

        let internal_map_update = InternalMapUpdate {
            rejected_outcomes: filter_update.rejections,
            discoveries: filter_update.discoveries,
            new_transf,
        };

        Ok(internal_map_update)
    }

    // WARN: LOCKS THE QUEUE RECEIVER; Also returns pending as long as the micromouse is stopped
    pub async fn notified_empty_queue(&self) {
        if *self.mode.lock().await == MicromouseMode::Stopped {
            info!(target: "comm/mng/cmd", "COMMAND PENDING; Micromouse Stopped");
            return pending().await;
        }
        loop {
            let mut queue_length_receiver = self.queue_length_receiver.lock().await;
            queue_length_receiver.changed().await.expect("Please no");
            debug!(target: "comm/mng", "NOTIFY QUEUE CHANGE");
            let val = queue_length_receiver.borrow_and_update();
            if *val < self.target_queue_length {
                debug!(target: "comm/mng", ">>> BELOW TARGET ({} < {})",*val, self.target_queue_length);
                break;
            }
        }
    }

    pub async fn update_queue_count(&self) {
        self.queue_length_sender
            .send(self.unconfirmed_cmd.lock().await.len())
            .expect("Stays const, shouldn't error");
        // let mut queue_length_receiver = self.queue_length_receiver.lock().await;
        // let val = queue_length_receiver.borrow_and_update();
        //
        // if *val < self.target_queue_length {
        //     // Re-updating to get the attention
        //     self.queue_length_sender.send(*val).expect("Stays const, shouldn't error");
        // }
    }

    /// Sets the current_cmd to none, returns the old one; Should NEVER return None
    async fn clear_current_command<'a>(
        &self,
        current_cmd: &mut MutexGuard<'a, Option<(FilteredCommandApplication<N>, CommandId)>>,
    ) -> Option<FilteredCommandApplication<N>> {
        debug!(target: "comm/mng/cmd", "CLEARING CURRENT COMMAND");
        debug!(target: "comm/mng/cmd", "UNCONFIRMEND CMD EMPTY");
        // let mut current_cmd = self.current_command.lock().await;
        let old_cmd = current_cmd.take();
        old_cmd.map(|x| x.0)
    }

    /// Checks whether the cmd_id contained in the response is a new one --> Would mean, that the
    /// previous command **has** to be finished and a new cmd started
    async fn update_current_command_id<'a>(
        &self,
        response_cmd_id: CommandId,
        current_cmd: &mut MutexGuard<'a, Option<(FilteredCommandApplication<N>, CommandId)>>,
    ) -> Result<(), MicromouseManagerError> {
        info!(target: "comm/mng/cmd", "UPDATING CURRENT CMD ID");
        // let (transformed_cmd, mut current_cmd) = self.current_command.lock().await;
        // let mut current_cmd = self.current_command.lock().await;
        if current_cmd.is_none() {
            debug!(target: "comm/mng/cmd", "NO CURRENT CMD REGISTERED");
            // Get the command we just started from the list of unconfirmed commands (commands that
            // were sent but have not yet sent any processing information) --> Should exist, error
            // otherwise

            let new_cmd = {
                let mut unconfirmed_cmd_queue = self.unconfirmed_cmd.lock().await;
                let new_cmd = unconfirmed_cmd_queue
                    .remove(&response_cmd_id)
                    .ok_or(MicromouseManagerError::CmdNotKnown(response_cmd_id))?;

                debug!(target: "comm/mng/cmd", "COMMAND QUEUE LEN = {}", unconfirmed_cmd_queue.len());
                // Queue decreased in length:
                self.queue_length_sender
                    .send(unconfirmed_cmd_queue.len())
                    .expect("Channels stay const");
                new_cmd
            };

            // Storing the starting state of the new command, so that we can easily calculate,
            // where the mouse currently is
            let new_cmd = FilteredCommandApplication::new(
                Some(self.current_world.read().await.clone()),
                new_cmd.cmd,
            );
            debug!(target: "comm/mng/cmd", "CONFIRMATION: STARTED NEW CMD");
            for outcome in new_cmd.potential_outcome_ids().potential_outcome_ids {
                debug!(target: "comm/mng/cmd", "POT. OUTCOME = {outcome:?}");
            }
            **current_cmd = Some((new_cmd, response_cmd_id));

            let expected_next_id = self
                .next_cmd_process_id
                .load(std::sync::atomic::Ordering::SeqCst);
            // let expected_next_id = last_processed_id + 1;
            if expected_next_id == response_cmd_id.0 {
                info!(target: "comm/mng/cmd", "Cmd Id matches next expected --> Next cmd; now = {expected_next_id}, next = {}", expected_next_id + 1);
                self.next_cmd_process_id
                    .store(expected_next_id + 1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            } else {
                error!(target: "comm/mng/cmd", "WRONG PROCESS ORDER: tried_starting = #{}, expected = #{expected_next_id}", response_cmd_id.0);
                // WARN: LIFE MUST GO ON (but really, this essentially silences the error, so that
                // it only triggers once, have to be aware of that)
                self.next_cmd_process_id
                    .store(response_cmd_id.0 + 1, std::sync::atomic::Ordering::SeqCst);
                Err(MicromouseManagerError::WrongProcessOrder {
                    expected: CommandId(expected_next_id),
                    received: response_cmd_id,
                })
            }
        } else {
            let current_cmd_id = current_cmd.as_ref().unwrap().1;
            debug!(target: "comm/mng/cmd", "CURRENTLY REGISTERED = {current_cmd_id}");

            if current_cmd_id == response_cmd_id {
                // No change; current command = new command
                Ok(())
            } else {
                // Life must go on
                // Pretend, that the next cmd we expect to see is the one after that
                self.next_cmd_process_id
                    .store(response_cmd_id.0 + 1, std::sync::atomic::Ordering::SeqCst);
                // Started a new command without exiting the previous one
                error!(target: "comm/mng/cmd", "MISSING CMD FINISH, TRIED STARTING NEW ONE");

                Err(MicromouseManagerError::CmdStartBeforeFinish {
                    new_cmd: response_cmd_id,
                    unfinished_cmd: current_cmd_id,
                })
            }
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

#[derive(Debug, Serialize)]
pub enum MicromouseEvent {
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

#[derive(Debug, Serialize, Deserialize)]
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
    WrongProcessOrder {
        expected: CommandId,
        received: CommandId,
    },
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

impl From<InternalMapUpdate> for Vec<MicromouseEvent> {
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
        vec
    }
}
