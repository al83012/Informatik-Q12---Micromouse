use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Instant,
};
use tokio_util::sync::CancellationToken;
use tungstenite::Message;

use crate::{
    comm::{
        micromouse_manager::MicromouseManagerError,
        websocket::{WsChannel, WsChannelConfig, WsChannelConnError},
    }, map::map::{CellDiscovery, WallDiscovery}, strategy::strategies::strategy_type::StrategyType, utils::nonempty::PotentiallyNonEmpty
};

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct DiscoveryMessage {
    pub cell_discoveries: Vec<CellDiscovery>,
    pub wall_discoveries: Vec<WallDiscovery>,
}

pub enum FrontendMessage {
    MicromouseManagerError(MicromouseManagerError),
    Debug(String),
}

pub enum FrontendResponse {
    NewStrategy{strategy_type: StrategyType},
    // New Strategy will set the next strategy to be used after the execution finishes
    // It will not be applied until the whole execution (to goal + back to 0,0) is finished or
    // cancel is sent (which will create a new strategy tree with this strategy after the old tree
    // finished execution)
    ResetAll, // Resetting Position, the strategy tree (from the point highest point that was
    // not already sent)
    Pause, // Stop sending commands, but be prepared to continue
    Cancel, // Cancel current execution (from the first non-sent branches onward, then apply the
    // current strategy from there on) --> Delete all the non-sent layers of the strategy tree;
    // After processing the last layer of the tree (the last sent commands) --> Take the world at
    // that step as the basis of a new strategy tree and start from there on
    Continue, // Continue the current strategy 
}

pub struct BatchedFrontendMessage(pub Vec<FrontendMessage>);

impl PotentiallyNonEmpty for DiscoveryMessage {
    fn is_empty(&self) -> bool {
        self.cell_discoveries.is_empty() && self.wall_discoveries.is_empty()
    }
}

pub struct FrontendConnectionManagerInternal {
    send_queue: Receiver<FrontendMessage>,
    first_element_send_time: Option<Instant>,
    batching_duration: Duration,
    cancellation_token: CancellationToken,
    websocket: WsChannel,
}

pub struct FrontendConnectionManager {
    cancellation_token: CancellationToken,
    send_queue: Sender<FrontendMessage>,
}

pub struct FrontendConnectionConfig {
    pub batching_duration: Duration,
    pub ws_channel_config: WsChannelConfig,
}

// pub struct

impl FrontendConnectionManager {
    pub async fn new(
        port: u16,
        config: FrontendConnectionConfig,
    ) -> Result<Self, WsChannelConnError> {
        let new_ws = WsChannel::new(config.ws_channel_config, port).await?;

        let (send_queue_sender, send_queue_receiver) = channel(16);

        let cancellation_token = CancellationToken::new();

        let internal = FrontendConnectionManagerInternal {
            batching_duration: config.batching_duration,
            send_queue: send_queue_receiver,
            first_element_send_time: None,
            cancellation_token: cancellation_token.clone(),
            websocket: new_ws,
        };

        tokio::task::spawn(async move {
            internal.handle_connection_loop().await;
        });

        let thread_connection = FrontendConnectionManager {
            cancellation_token,
            send_queue: send_queue_sender,
        };

        Ok(thread_connection)
    }
}

impl FrontendConnectionManagerInternal {
    pub async fn handle_connection_loop(&self) {
        loop {
            if self.first_element_send_time.is_some()
                && self.first_element_send_time.unwrap().elapsed() > self.batching_duration {
                    // TODO: SEND ALL
                    continue;
                }
            tokio::select! {
                batch = self.batch() => {
                    self.send_batch(batch).await;
                }
                msg = self.next_read() => {
                    self.handle_read(msg).await;
                }
                _msg = self.next_send() => {
                    self.register_send();
                }
            }
        }
    }

    pub async fn next_read(&self) -> Message {
        todo!("Only one state in the future; should never interrupt execution halfway through")
    }
    pub async fn next_send(&self) -> FrontendMessage {
        todo!("Get the next message to be sent via the channel")
    }
    pub async fn batch(&self) -> BatchedFrontendMessage {
        todo!("Returns only once the time elapsed since first_element_send_time exceeds batching_duration")
    }
    pub async fn handle_read(&self, _msg: Message) {
        todo!("Actually handles all the parsing; The read_next fn is only allowed to contain one substep")
    }
    pub async fn send_batch(&self, _batch: BatchedFrontendMessage) {
        todo!("Send the batch of frontend messages as the actual .batch()-method may only contain one substep")
    }
    pub async fn register_send(&self) {
        todo!("Set the first_element_send_time if not already there");
    }
}

impl Drop for FrontendConnectionManager {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
