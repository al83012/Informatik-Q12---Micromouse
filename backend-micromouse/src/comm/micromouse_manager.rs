use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, AtomicUsize},
};

use tokio::sync::Mutex;

use crate::{
    comm::{
        micromouse_message::{
            Command, CommandFinishedMessage, CommandId, CommandMessage, FormatError, MicromouseResponse
        },
        website::DiscoveryMessage,
        websocket::{WsChannel, WsChannelConfig, WsChannelConnError},
    }, map::Map, nonempty::NonEmptyVec, position::MouseTransform, world_data::WorldData
};

pub struct MicromouseManager<const N: usize> {
    channel: WsChannel,
    next_cmd_id: AtomicU32,
    unfinished_messages: Mutex<HashMap<CommandId, CommandMessage>>,
    mode: MicromouseMode,
    /// 0.0 to 1.0
    battery: f32,
    world_data: WorldData<N>,
    map: Map<N>,
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
        todo!("Add to command queue");
        Ok(cmd_id)
    }

    // Returns boolean --> true = was resent, false = already finished, exited queue
    async fn resend(&self, cmd_id: CommandId) -> bool {
        if let Some(msg) = self.unfinished_messages.lock().await.get(&cmd_id) {
            self.channel.send(msg.into()).await;
            true
        } else {
            false
        }
    }

    /// WARN: Even when the event cannot be handled: Polling the next-function is necessary for the
    /// communication to continue (even though the channel will spin up a separate thread to keep
    /// the connection alive, it won't handle desyncs and the like)
    pub async fn next(&self) ->  Result<NonEmptyVec<MicromouseEvent<N>>, MicromouseManagerError> {

        loop {
            let next_response = &self.channel.read().await;
            if next_response.is_none() {
                return Err(MicromouseManagerError::ConnectionClosedPermanently);
            }
            let next_response: MicromouseResponse = next_response.as_ref().unwrap().to_string().try_into()?;
            match next_response {
                MicromouseResponse::Debug(msg) =>return Ok(NonEmptyVec::one(MicromouseEvent::DebugMessage(msg))),
                MicromouseResponse::Measurement(measurement_message) => todo!(),
                MicromouseResponse::CommandFinished(command_finished_message) => todo!(),
                MicromouseResponse::Desync(command_ids) => todo!(),
                MicromouseResponse::Stop => todo!(),
                MicromouseResponse::Restart => todo!(),
                MicromouseResponse::Battery(_) => todo!(),
            }
            




            todo!(
            "Processing all the events which can be handled internally --> 
Automatically adjust the position after each substep, so that measurements can be transformed and applied to the map;
(Only cause event if there is a nonempty discovery message);
Update the map (like clearing it), also with Stop and Restart;
Clear commands that are sent while the micromouse is stopped / error on send-attempt;
Clear queue etc. on stop;

On Desync: automatically resend all the commands; keep lock to prevent writing new tasks
"
        )
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

}


impl From<FormatError<MicromouseResponse>> for MicromouseManagerError {
    fn from(value: FormatError<MicromouseResponse>) -> Self {
        Self::UnknownResponse(value)
    }
}
