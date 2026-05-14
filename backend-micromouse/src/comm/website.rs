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
use tracing::{error, info, instrument, Instrument};
use tungstenite::{Message, Utf8Bytes};

use crate::{
    comm::{
        micromouse_manager::{MicromouseEvent, MicromouseManagerError},
        websocket::{WsChannel, WsChannelConfig, WsChannelConnError},
    },
    map::map::{CellDiscovery, WallDiscovery},
    strategy::{dyn_strategy_tree::StrategyChangeCommand, strategy_tree::StrategyTreeError},
    utils::{hyperlink_logging::process_span, nonempty::PotentiallyNonEmpty},
};

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct DiscoveryMessage {
    pub cell_discoveries: Vec<CellDiscovery>,
    pub wall_discoveries: Vec<WallDiscovery>,
}

#[derive(Serialize, Debug)]
pub enum FrontendMessage {
    MicromouseEvent(MicromouseEvent),
    StrategyTreeError(StrategyTreeError),
    Debug(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum FrontendResponse<const N: usize> {
    StrategyChange(StrategyChangeCommand<N>),
    Pause,
    Continue,
}

#[derive(Serialize)]
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

pub struct FrontendConnectionManagerInternal<const N: usize> {
    send_queue: Receiver<FrontendMessage>,
    read_queue: Sender<FrontendResponse<N>>,
    first_element_send_time: Option<Instant>,
    batching_duration: Duration,
    cancellation_token: CancellationToken,
    websocket: WsChannel,
    send_batch: Vec<FrontendMessage>,
}

pub struct FrontendManager<const N: usize> {
    cancellation_token: CancellationToken,
    send_queue: Sender<FrontendMessage>,
    read_queue: Receiver<FrontendResponse<N>>,
}

#[derive(Default, Debug)]
pub struct FrontendConnectionConfig {
    pub batching_duration: Duration,
    pub ws_channel_config: WsChannelConfig,
}

// pub struct

impl<const N: usize> FrontendManager<N> {
    #[instrument(name = "new FrontendManager")]
    pub async fn new(
        port: u16,
        config: FrontendConnectionConfig,
    ) -> Result<Self, WsChannelConnError> {
        let new_ws = WsChannel::new(config.ws_channel_config, port).await?;

        let (send_queue_sender, send_queue_receiver) = channel(16);
        let (read_sender, read_receiver) = channel(16);

        let cancellation_token = CancellationToken::new();

        info!(target: "comm/webs", "Creating Internal Frontend Connection Handler");

        let mut internal = FrontendConnectionManagerInternal {
            batching_duration: config.batching_duration,
            send_queue: send_queue_receiver,
            first_element_send_time: None,
            cancellation_token: cancellation_token.clone(),
            websocket: new_ws,
            read_queue: read_sender,
            send_batch: vec![],
        };
        info!(target: "comm/webs", "Created Internal Frontend Connection Handler");

        tokio::task::spawn(
            async move {
                internal.handle_connection_loop().await;
            }
            .instrument(process_span("webs_conn_loop")),
        );
        info!(target: "comm/webs", "Spawned frontend communication thread");

        let thread_connection = FrontendManager {
            cancellation_token,
            send_queue: send_queue_sender,
            read_queue: read_receiver,
        };

        Ok(thread_connection)
    }
}

impl<const N: usize> FrontendConnectionManagerInternal<N> {
    #[instrument(name = "frontend_handle_conn_loop", skip(self))]
    pub async fn handle_connection_loop(&mut self) {
        loop {
            let first_element_send_time_clone = self.first_element_send_time.clone();
            let batching_duration_clone = self.batching_duration.clone();
            let ws = &self.websocket;
            let send_queue = &mut self.send_queue;
            let cancellation = self.cancellation_token.clone();
            tokio::select! {
                _ = cancellation.cancelled() => {
                    info!(target: "comm/webs", "CANCELLED FRONTEND INTERNAL LOOP");
                    break;
                }
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

    #[instrument(
        name = "next_read",
        fields(description = "Read new message from frontend; Not processing yet"),
        skip(read_source)
    )]
    pub async fn next_read(read_source: &WsChannel) -> Option<Message> {
        read_source.read().await
        // todo!("Only one state in the future; should never interrupt execution halfway through")
    }
    #[instrument(
        name = "next_send",
        fields(description = "Read next msg to send to frontend from queue; Not processing yet"),
        skip(send_queue)
    )]
    pub async fn next_send(send_queue: &mut Receiver<FrontendMessage>) -> FrontendMessage {
        send_queue
            .recv()
            .await
            .expect("Channel should always be open")
    }
    #[instrument(
        name = "batch_ready",
        fields(
            description = "Check whether an element in the queue is older than the maximum desired batch-interval"
        )
    )]
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
    pub fn parse_msg(msg: &str) -> serde_json::Result<FrontendResponse<N>> {
        serde_json::de::from_str(msg)
    }

    #[instrument(
        name = "propagate_read",
        fields(description = "Parse the message and add it to the read-queue"),
        skip(self)
    )]
    pub async fn propagate_read(&self, msg: Option<Message>) {
        // Parses and sends to the ConnectionHandler
        let Some(msg) = msg else {
            error!(target: "comm/webs", "Propagating empty msg");
            return;
        };
        let Message::Text(msg) = msg else {
            info!(target: "comm/webs", "Non-Text-Msg");
            return;
        };

        let Ok(parsed) = Self::parse_msg(msg.to_string().as_str()) else {
            error!(target: "comm/webs", "Invalid message: {msg}");
            return;
        };
        // let parsed_msg = todo!("Parse command or cmd close");
        self.read_queue
            .send(parsed)
            .await
            .expect("Channel should always be open");
    }
    #[instrument(
        name = "send_batch",
        fields(
            description = "Empty the current send-queue, pack all elements to a batch and send them"
        ),
        skip(self)
    )]
    pub async fn send_batch(&mut self) {
        let mut empty = vec![];
        std::mem::swap(&mut empty, &mut self.send_batch);
        self.websocket
            .send(BatchedFrontendMessage(empty).into())
            .await;
        self.first_element_send_time = None;
    }
    #[instrument(
        name = "register_send",
        fields(description = "Add message to the send-queue and record its time"),
        skip(self)
    )]
    pub async fn register_send(&mut self, msg: FrontendMessage) {
        if self.first_element_send_time.is_none() {
            self.first_element_send_time = Some(Instant::now());
        }
        self.send_batch.push(msg);
    }
}

impl<const N: usize> Drop for FrontendManager<N> {
    fn drop(&mut self) {
        info!(target: "comm/webs", "DROPPED FRONTEND MANAGER");
        self.cancellation_token.cancel();
    }
}

impl<const N: usize> FrontendManager<N> {
    #[instrument(
        name = "next_read",
        fields(description = "Read next frontend response"),
        skip(self)
    )]
    pub async fn next_read(&mut self) -> FrontendResponse<N> {
        let msg = self
            .read_queue
            .recv()
            .await
            .expect("Channel should not be dropped during execution");
        info!(target: "comm/webs", "{msg:?}");
        msg
    }

    #[instrument(
        name = "send",
        fields(description = "Send msg (and potentially batch it first)"),
        skip(self)
    )]
    pub async fn send(&mut self, msg: FrontendMessage) {
        self.send_queue
            .send(msg)
            .await
            .expect("Channel should not be dropped during execution");
    }
}
