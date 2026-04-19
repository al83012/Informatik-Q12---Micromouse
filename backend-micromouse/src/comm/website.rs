use std::{future, time::Duration};

use futures_util::future::pending;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{
        broadcast,
        mpsc::{channel, Receiver, Sender},
    },
    time::{sleep_until, Instant},
};
use tokio_util::sync::CancellationToken;
use tungstenite::{Message, Utf8Bytes};

use crate::{
    comm::{
        micromouse_manager::MicromouseManagerError,
        websocket::{WsChannel, WsChannelConfig, WsChannelConnError},
    },
    map::map::{CellDiscovery, WallDiscovery},
    strategy::strategies::strategy_type::StrategyType,
    utils::nonempty::PotentiallyNonEmpty,
};

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct DiscoveryMessage {
    pub cell_discoveries: Vec<CellDiscovery>,
    pub wall_discoveries: Vec<WallDiscovery>,
}

#[derive(Serialize, Deserialize)]
pub enum FrontendMessage {
    MicromouseManagerError(MicromouseManagerError),
    Debug(String),
}

pub enum FrontendResponse {
    NewStrategy { strategy_type: StrategyType },
    // New Strategy will set the next strategy to be used after the execution finishes
    // It will not be applied until the whole execution (to goal + back to 0,0) is finished or
    // cancel is sent (which will create a new strategy tree with this strategy after the old tree
    // finished execution)
    ResetAll, // Resetting Position, the strategy tree (from the point highest point that was
    // not already sent)
    Pause,  // Stop sending commands, but be prepared to continue
    Cancel, // Cancel current execution (from the first non-sent branches onward, then apply the
    // current strategy from there on) --> Delete all the non-sent layers of the strategy tree;
    // After processing the last layer of the tree (the last sent commands) --> Take the world at
    // that step as the basis of a new strategy tree and start from there on
    Continue, // Continue the current strategy
}

#[derive(Serialize, Deserialize)]
pub struct BatchedFrontendMessage(pub Vec<FrontendMessage>);

impl Into<Message> for BatchedFrontendMessage {
    fn into(self) -> Message {
        let msg = serde_json::to_string_pretty(&self).expect("SERIALIZATION FAILED");
        Message::Text(Utf8Bytes::from(msg))
    }
}

impl PotentiallyNonEmpty for DiscoveryMessage {
    fn is_empty(&self) -> bool {
        self.cell_discoveries.is_empty() && self.wall_discoveries.is_empty()
    }
}

pub struct FrontendConnectionManagerInternal {
    send_queue: Receiver<FrontendMessage>,
    read_queue: broadcast::Sender<Message>,
    first_element_send_time: Option<Instant>,
    batching_duration: Duration,
    cancellation_token: CancellationToken,
    websocket: WsChannel,
    send_batch: Vec<FrontendMessage>,
}

pub struct FrontendConnectionManager {
    cancellation_token: CancellationToken,
    send_queue: Sender<FrontendMessage>,
    read_queue: broadcast::Receiver<Message>,
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
        let (read_sender, read_receiver) = broadcast::channel(16);

        let cancellation_token = CancellationToken::new();

        let mut internal = FrontendConnectionManagerInternal {
            batching_duration: config.batching_duration,
            send_queue: send_queue_receiver,
            first_element_send_time: None,
            cancellation_token: cancellation_token.clone(),
            websocket: new_ws,
            read_queue: read_sender,
            send_batch: vec![],
        };

        tokio::task::spawn(async move {
            internal.handle_connection_loop().await;
        });

        let thread_connection = FrontendConnectionManager {
            cancellation_token,
            send_queue: send_queue_sender,
            read_queue: read_receiver,
        };

        Ok(thread_connection)
    }
}

impl FrontendConnectionManagerInternal {
    pub async fn handle_connection_loop(&mut self) {
        loop {
            let first_element_send_time_clone = self.first_element_send_time.clone();
            let batching_duration_clone = self.batching_duration.clone();
            let ws = &self.websocket;
            let send_queue = &mut self.send_queue;
            tokio::select! {
                _ = Self::batch_ready(first_element_send_time_clone, batching_duration_clone) => {
                    self.send_batch().await;
                }
                msg = Self::next_read(ws) => {
                    self.propagate_read(msg).await;
                }
                msg = Self::next_send(send_queue) => {
                    self.register_send(msg).await;
                }
            }
        }
    }

    pub async fn next_read(read_source: &WsChannel) -> Option<Message> {
        read_source.read().await
        // todo!("Only one state in the future; should never interrupt execution halfway through")
    }
    pub async fn next_send(send_queue: &mut Receiver<FrontendMessage>) -> FrontendMessage {
        send_queue
            .recv()
            .await
            .expect("Channel should always be open")
    }
    pub async fn batch_ready(
        first_element_send_time: Option<Instant>,
        batching_duration: Duration,
    ) {
        if let Some(send_time) = first_element_send_time {
            let wait_until = send_time + batching_duration;
            sleep_until(wait_until).await;
        } else {
            future::pending().await
        }
    }
    pub async fn propagate_read(&self, msg: Option<Message>) {
        // Parses and sends to the ConnectionHandler
        let parsed_msg = todo!("Parse command or cmd close");
        self.read_queue.send(parsed_msg);
    }
    pub async fn send_batch(&mut self) {
        let mut empty = vec![];
        std::mem::swap(&mut empty, &mut self.send_batch);
        self.websocket
            .send(BatchedFrontendMessage(empty).into())
            .await;
    }
    pub async fn register_send(&mut self, msg: FrontendMessage) {
        if self.first_element_send_time.is_none() {
            self.first_element_send_time = Some(Instant::now());
        }
        self.send_batch.push(msg);
    }
}

impl Drop for FrontendConnectionManager {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
