use std::{
    collections::HashMap,
    ops::Deref,
    sync::atomic::{AtomicU32, AtomicUsize},
};

use tokio::sync::Mutex;

use crate::{
    comm::{
        micromouse_message::{
            Command, CommandFinishedMessage, CommandId, CommandMessage, FormatError, InterruptStep,
            MicromouseResponse, StepNum, TransformedCommand, TransformedMovement,
        },
        website::DiscoveryMessage,
        websocket::{WsChannel, WsChannelConfig, WsChannelConnError},
    },
    map::map::Map,
    utils::nonempty::NonEmptyVec,
    transform::position::MouseTransform,
    map::world_data::WorldData,
};

pub struct MicromouseManager<const N: usize> {
    channel: WsChannel,
    next_cmd_id: AtomicU32,
    unconfirmed_cmd: Mutex<HashMap<CommandId, CommandMessage>>,
    mode: MicromouseMode,
    current_command: Mutex<Option<(TransformedMovement, CommandId)>>,
    /// 0.0 to 1.0
    // battery: f32,
    current_world_data: Mutex<WorldData<N>>,
}

impl<const N: usize> MicromouseManager<N> {
    pub async fn new() -> Result<Self, WsChannelConnError> {
        todo!();
        // Ok(Self {
        //     channel: WsChannel::new(WsChannelConfig::default(), 9001).await?,
        //     next_cmd_id: AtomicU32::new(0),
        //     unfinished_messages: Mutex::new(HashMap::new()),
        // })
    }

    pub async fn send_command(&self, cmd: Command) -> Result<CommandId, CommandSendError> {
        if self.mode == MicromouseMode::Stopped {
            return Err(CommandSendError::StoppedExecution);
        }
        let cmd_id = CommandId(
            self.next_cmd_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );
        let msg = CommandMessage { cmd, cmd_id };
        self.channel.send((&msg).into()).await;
        self.unconfirmed_cmd.lock().await.insert(cmd_id, msg);
        // todo!("Add to command queue");
        Ok(cmd_id)
    }

    // Returns boolean --> true = was resent, false = already finished, exited queue
    async fn resend(&self, cmd_id: CommandId) -> bool {
        if let Some(msg) = self.unconfirmed_cmd.lock().await.get(&cmd_id) {
            self.channel.send(msg.into()).await;
            true
        } else {
            false
        }
    }

    /// WARN: Even when the event cannot be handled: Polling the next-function is necessary for the
    /// communication to continue (even though the channel will spin up a separate thread to keep
    /// the connection alive, it won't handle desyncs and the like)
    pub async fn next(&self) -> Result<NonEmptyVec<MicromouseEvent<N>>, MicromouseManagerError> {
        loop {
            let next_response = &self.channel.read().await;
            if next_response.is_none() {
                return Err(MicromouseManagerError::ConnectionClosedPermanently);
            }
            let next_response: MicromouseResponse =
                next_response.as_ref().unwrap().to_string().try_into()?;
            match next_response {
                MicromouseResponse::Debug(msg) => {
                    return Ok(NonEmptyVec::one(MicromouseEvent::DebugMessage(msg)))
                }
                MicromouseResponse::Measurement(measurement_message) => {
                    // Check whether command is new
                    self.update_current_command_id(measurement_message.from_cmd)
                        .await?;
                    todo!("Update current transform, update map")
                }
                MicromouseResponse::CommandFinished(command_finished_message) => {
                    todo!("Update current transform, delete current cmd");
                },
                MicromouseResponse::Desync(command_ids) => todo!(),
                MicromouseResponse::Stop => todo!(),
                MicromouseResponse::Restart => todo!(),
                MicromouseResponse::Battery(_) => todo!(),
            }

            //             todo!(
            //             "Processing all the events which can be handled internally -->
            // Automatically adjust the position after each substep, so that measurements can be transformed and applied to the map;
            // (Only cause event if there is a nonempty discovery message);
            // Update the map (like clearing it), also with Stop and Restart;
            // Clear commands that are sent while the micromouse is stopped / error on send-attempt;
            // Clear queue etc. on stop;
            //
            // On Desync: automatically resend all the commands; keep lock to prevent writing new tasks
            // "
            //         )
        }
    }

    async fn update_current_transform(
        &self,
        step_number: StepNum,
    ) -> Result<(), MicromouseManagerError> {
        let mut current_cmd = self.current_command.lock().await;

        let (transf_mov, id) = current_cmd
            .as_mut()
            .ok_or(MicromouseManagerError::MeasurementWithoutAssociatedCmd)?;
        if step_number > transf_mov.max_step_count() as u32 {
            return Err(MicromouseManagerError::CmdTooLong(*id));
        }
        let new_pos = transf_mov
            .at_step(step_number as usize)
            .ok_or(MicromouseManagerError::ImpossiblePosition(*id))?;
        self.current_world_data.lock().await.mouse = new_pos;
        Ok(())
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
                TransformedMovement::new(
                    new_cmd.cmd.ty,
                    self.current_world_data.lock().await.mouse,
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
            Err(MicromouseManagerError::CmdStartBeforeFinish {
                new_cmd: response_cmd_id,
                unfinished_cmd: current_cmd_id,
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MicromouseMode {
    Stopped,
    Running,
}

pub enum MicromouseEvent<const N: usize> {
    UpdatePosition,
    UpdatedMap(Map<N>, DiscoveryMessage),
    FinishedCommand {
        cmd_id: CommandId,
        require_new: bool,
    },
    Stop,
    Restart,
    Error(MicromouseManagerError),
    DebugMessage(String),
}

pub enum CommandSendError {
    /// The strategy was manually stopped, no command should be sent, it will be voided
    StoppedExecution,
}

pub enum MicromouseManagerError {
    ConnectionClosedPermanently,
    UnknownResponse(FormatError<MicromouseResponse>),
    CmdConfirmThenReqested,
    CmdStartBeforeFinish {
        new_cmd: CommandId,
        unfinished_cmd: CommandId,
    },
    CmdNotKnown(CommandId),
    MeasurementWithoutAssociatedCmd,
    CmdTooLong(CommandId),
    ImpossiblePosition(CommandId),
}

impl From<FormatError<MicromouseResponse>> for MicromouseManagerError {
    fn from(value: FormatError<MicromouseResponse>) -> Self {
        Self::UnknownResponse(value)
    }
}
