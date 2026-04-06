use std::{
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use futures_util::lock::Mutex;
use tracing::{error, info};
use tokio::{
    select,
    sync::mpsc::{Receiver, Sender},
    time::{self, interval_at, Instant},
};
use tokio_util::sync::CancellationToken;

use crate::comm::wifi_channel::{WifiChannel, WifiConnError};

pub struct HeartbeatWifiChannel {
    // There is no specific read-request, as we are reading continuously
    read_response: Receiver<Result<String, WifiConnError>>,
    send_request: Sender<String>,
    send_response: Receiver<Result<(), WifiConnError>>,

    cancellation_token: CancellationToken,
}

pub struct HeartbeatWifiChannelConfig {
    send_interval: Duration,
    valid_response_interval: Duration,
    msg_to: String,
    expected_response: String,
    delim: u8,
    buffer_size: usize,
}

impl HeartbeatWifiChannelConfig {
    pub fn new(
        send_interval: Duration,
        valid_response_interval: Duration,
        msg_to: String,
        expected_response: String,
        delim: u8,
        buffer_size: usize,
    ) -> Option<Self> {
        // I do not want to bother with handling a whole buffer of alive-messages
        if send_interval.as_millis() < valid_response_interval.as_millis() {
            return None;
        }

        Some(Self {
            send_interval,
            valid_response_interval,
            msg_to,
            expected_response,
            delim,
            buffer_size,
        })
    }
}

impl Default for HeartbeatWifiChannelConfig {
    fn default() -> Self {
        Self {
            send_interval: Duration::from_millis(1000),
            valid_response_interval: Duration::from_millis(200),
            msg_to: "ALIVE".to_owned(),
            expected_response: "CONFIRM-ALIVE".to_owned(),
            delim: b'$',
            buffer_size: 64,
        }
    }
}

impl HeartbeatWifiChannel {
    pub fn new(mut channel: WifiChannel, config: HeartbeatWifiChannelConfig) -> Self {
        // let (read_request, read_request_recv) = tokio::sync::mpsc::channel(config.buffer_size);
        let (read_response_sender, read_response) = tokio::sync::mpsc::channel(config.buffer_size);

        let (send_request, mut send_request_recv) =
            tokio::sync::mpsc::channel::<String>(config.buffer_size);
        let (send_response_sender, send_response) =
            tokio::sync::mpsc::channel::<Result<(), WifiConnError>>(config.buffer_size);

        let cancellation_token = CancellationToken::new();

        let cancellation_token_recv = cancellation_token.clone();

        let mut send_interval = time::interval(config.send_interval);

        // let (sync_send, sync_recv) = tokio::sync::mpsc::channel(2);

        let mut last_send = Instant::now();
        let mut recv_ack = true;

        let alive_msg = format!(
            "{}{}",
            config.msg_to,
            String::from_utf8(vec![config.delim]).unwrap()
        );

        let expected_resp_msg = format!(
            "{}{}",
            config.expected_response,
            String::from_utf8(vec![config.delim]).unwrap()
        );

        tokio::spawn(async move {
            loop {
                info!(target: "comm", ">>>>>>>>>>>>>>> Processing loop");
                tokio::select! {

                    // Ending the loop if the associated object is dropped
                    _ = cancellation_token_recv.cancelled() => {
                        info!(target: "comm", "KILLED HEARTBEAT CHANNEL");
                        break;
                    },

                    // Sending the heartbeat at intervals
                    _ = send_interval.tick() => {

                        info!(target: "comm", "HEARTBEAT TICK");
                        let elapsed = last_send.elapsed();
                        if !recv_ack && elapsed > config.valid_response_interval {
                            error!(target: "comm", "HEARTBEAT TOO LATE (next tick) ({elapsed:?})");
                            // WARN: Cannot actually reconnect if current connection is still alive
                            /*if let Err(e) = channel.reconnect().await {
                                send_response_sender.send(Err(e)).await.expect("Error while sending via mpsc channel");
                            }*/
                        }
                        info!(target: "comm", "SENT ALIVE \"{alive_msg}\"");
                        if let Err(e) = unsafe {channel.send_maybe_disconnect(&alive_msg).await} {
                            send_response_sender.send(Err(e)).await.expect("Error while sending via mpsc channel");
                            continue;
                        }
                        last_send = Instant::now();
                        recv_ack = false;

                    },


                    // Always reading if there is nothing else to do
                    read_res = channel.read_until_delim(config.delim) => {
                        info!(target: "comm", "READ TICK");
                        match read_res {
                            Ok(msg) => {
                                info!(target: "comm", "READ MSG = {msg}");
                                if msg == expected_resp_msg {
                                    let elapsed = last_send.elapsed();

                                        info!(target: "comm", "HEARTBEAT RECEIVED ({elapsed:?})");
                                    if elapsed > config.valid_response_interval {

                                        error!(target: "comm", "HEARTBEAT TOO LATE");

                                        // WARN: Cannot actually reconnect if current connection is still alive
                                        /*if let Err(e) = channel.reconnect().await {
                                            read_response_sender.send(Err(e)).await.expect("Error while sending via mpsc channel");
                                        }*/
                                    }
                                    recv_ack = true;
                                }
                            },
                            Err(e) => read_response_sender.send(Err(e)).await.expect("Error while sendig via mpsc channel"),
                        }
                    },
                    send_task = send_request_recv.recv() => {
                        info!(target: "comm", "SEND TICK");
                        let elapsed = last_send.elapsed();
                        if !recv_ack && elapsed > config.valid_response_interval {
                            error!(target: "comm", "HEARTBEAT TOO LATE (next tick) ({elapsed:?})");

                            // WARN: Cannot actually reconnect if current connection is still alive
                            /*if let Err(e) = channel.reconnect().await {
                                send_response_sender.send(Err(e)).await.expect("Error while sending via mpsc channel");
                            }*/
                        }
                        if send_task.is_none() {
                            continue;
                        }
                        if let Err(e) = unsafe {channel.send_maybe_disconnect(&send_task.unwrap()).await}{
                            send_response_sender.send(Err(e)).await.expect("Error while sending via mpsc channel");
                        }

                    },
                }
            }
        });

        Self {
            read_response,
            send_response,
            cancellation_token,
            send_request,
        }
    }

    pub async fn read(&mut self) -> Result<String, WifiConnError> {
        self.read_response
            .recv()
            .await
            .expect("Error while reading from mpsc channel")
    }

    pub async unsafe fn send_maybe_disconnect(&mut self, msg: &str) -> Result<(), WifiConnError> {
        self.send_request
            .send(msg.to_owned())
            .await
            .expect("Error while sending via mpsc channel");
        self.send_response
            .recv()
            .await
            .expect("Error while reading from mpsc channel")
    }
}

impl Drop for HeartbeatWifiChannel {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
