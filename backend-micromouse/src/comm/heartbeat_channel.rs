use std::{
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use futures_util::lock::Mutex;
use log::{error, info};
use tokio::{select, sync::mpsc::Receiver, time::Instant};
use tokio_util::sync::CancellationToken;

use crate::comm::wifi_channel::{WifiChannel, WifiConnError};

pub struct HeartbeatWifiChannel {
    channel: Arc<Mutex<WifiChannel>>,
    received_ack: Arc<AtomicBool>,
    read_channel: Receiver<Result<String, WifiConnError>>,
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
            valid_response_interval: Duration::from_millis(100),
            msg_to: "ALIVE".to_owned(),
            expected_response: "CONFIRM-ALIVE".to_owned(),
            delim: b'$',
            buffer_size: 64,
        }
    }
}

impl HeartbeatWifiChannel {
    pub fn new(channel: WifiChannel, config: HeartbeatWifiChannelConfig) -> Self {
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let periodic_writer_token = cancellation_token.clone();
        let reader_token = cancellation_token.clone();

        let channel = Arc::new(Mutex::new(channel));
        let last_send = Arc::new(Mutex::new(Instant::now()));
        let last_send_writer = last_send.clone();
        let last_send_reader = last_send.clone();

        let interval_send_channel = channel.clone();
        let greedy_read_channel = channel.clone();

        let received_ack = Arc::new(AtomicBool::new(false));
        let received_ack_completer = received_ack.clone();

        let (read_sender, read_receiver) = tokio::sync::mpsc::channel(config.buffer_size);

        let error_sender = read_sender.clone();

        tokio::spawn(async move {
            loop {
                {
                    let mut channel = interval_send_channel.lock().await;
                    if let Err(e) = unsafe { channel.send_maybe_disconnect(&config.msg_to).await } {
                        error_sender
                            .send(Err(e))
                            .await
                            .expect("Error while writing to mpsc channel");
                        return;
                    }

                    *last_send_writer.lock().await = Instant::now();
                    // sync_sender.send(0).await.expect("Error while writing to mpsc channel");
                    info!(target: "comm", "ALIVE");

                    select! {
                        _ = tokio::time::sleep(config.send_interval) => {},
                        _ = periodic_writer_token.cancelled() => {
                            info!(target: "comm", "KILLED periodic writer");
                            break;}
                    }
                }
            }
        });

        tokio::spawn(async move {
            loop {
                if reader_token.is_cancelled() {
                    info!(target: "comm", "KILLED reader");
                    break;
                }
                info!(target: "comm", "CONSTANT READ");

                let mut channel = greedy_read_channel.lock().await;

                let read = channel.read_until_delim(config.delim).await;

                if let Ok(msg) = &read {
                    if msg == &config.expected_response {
                        received_ack_completer.store(true, Ordering::Relaxed);
                        info!(target: "comm", "RECV ALIVE ACK");
                    }
                }

                read_sender
                    .send(read)
                    .await
                    .expect("Error while writing to mpsc channel");

                if received_ack_completer.load(Ordering::Relaxed)
                    && last_send_reader.lock().await.elapsed() > config.valid_response_interval
                {
                    error!(target: "comm", "DISCONNECTED Heartbeat not received on time");
                    if let Err(e) = channel.reconnect().await {
                        read_sender
                            .send(Err(e))
                            .await
                            .expect("Error while writing to mpsc channel");
                    }
                }
            }
        });

        Self {
            channel,
            received_ack,
            read_channel: read_receiver,
            cancellation_token,
        }
    }

    pub async fn read(&mut self) -> Result<String, WifiConnError> {
        self.read_channel
            .recv()
            .await
            .expect("Error while reading from mpsc channel")
    }

    pub async fn send(
        &mut self,
        msg: &str,
        test_read_delim: u8,
        error_search_time: Duration,
    ) -> Result<(), WifiConnError> {
        self.channel
            .lock()
            .await
            .send(msg, test_read_delim, error_search_time)
            .await
    }

    pub async unsafe fn send_maybe_disconnect(&mut self, msg: &str) -> Result<(), WifiConnError> {
        self.channel.lock().await.send_maybe_disconnect(msg).await
    }
}
