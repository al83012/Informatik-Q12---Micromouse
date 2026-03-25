use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, AtomicUsize},
};

use tokio::sync::Mutex;

use crate::{
    comm::{
        micromouse_message::{
            Command, CommandFinishedMessage, CommandId, CommandMessage, MicromouseResponse,
        }, website::DiscoveryMessage, websocket::{WsChannel, WsChannelConfig, WsChannelConnError}
    },
    map::Map,
    position::MouseTransform,
};

pub struct MicromouseManager<const N: usize> {
    channel: WsChannel,
    next_cmd_id: AtomicU32,
    unfinished_messages: Mutex<HashMap<CommandId, CommandMessage>>,
    mode: MicromouseMode,
    /// 0.0 to 1.0
    battery: f32,
    micromouse_position: MouseTransform,
    map: Map<N>,
}

impl<const N: usize> MicromouseManager<N> {
    pub async fn new() -> Result<Self, WsChannelConnError> {
        todo!();
        Ok(Self {
            channel: WsChannel::new(WsChannelConfig::default(), 9001).await?,
            next_cmd_id: AtomicU32::new(0),
            unfinished_messages: Mutex::new(HashMap::new()),
        })
    }

    pub async fn send_command(&self, cmd: Command) -> CommandId {
        let cmd_id = CommandId(
            self.next_cmd_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );
        let msg = CommandMessage { cmd, cmd_id };
        self.channel.send((&msg).into()).await;
        todo!("Add to command queue");
        cmd_id
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

    pub async fn next(&self) -> Vec<MicromouseEvent<N>> {
        let next_response = &self.channel.read().await;

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

pub enum MicromouseMode {
    Stopped,
    Running,
}

pub enum MicromouseEvent<const N: usize> {
    UpdatePosition,
    UpdatedMap(Map<N>, DiscoveryMessage),
    FinishedCommand{cmd_id: CommandId, require_new: bool},
    Stop,
    Restart,
}
