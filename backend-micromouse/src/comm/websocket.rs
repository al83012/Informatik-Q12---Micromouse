use std::{net::SocketAddr, time::Duration};

use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
    time::{self, Instant, Interval},
};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_util::{bytes::Buf, sync::CancellationToken};
use tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    Bytes, Error, Message, Utf8Bytes,
};

use crate::comm::{ChannelConnConfig, ChannelConnError};

pub struct WsChannel {
    // There is no specific read-request, as we are reading continuously
    read_recv: Receiver<Message>,
    e_recv: Receiver<WsChannelConnError>,
    send_request_sender: Sender<Message>,

    cancellation_token: CancellationToken,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum WsChannelMode {
    Stabilize,
    Stable,
}

#[derive(Debug)]
pub enum WsChannelConnError {
    ChannelConnError(ChannelConnError),
    WsConnError(tungstenite::Error),
}

impl From<tungstenite::Error> for WsChannelConnError {
    fn from(value: tungstenite::Error) -> Self {
        Self::WsConnError(value)
    }
}
impl From<ChannelConnError> for WsChannelConnError {
    fn from(value: ChannelConnError) -> Self {
        Self::ChannelConnError(value)
    }
}
impl From<std::io::Error> for WsChannelConnError {
    fn from(value: std::io::Error) -> Self {
        ChannelConnError::from(value).into()
    }
}

#[derive(Debug)]
pub struct WsChannelConfig {
    buffer_size: usize,
    conn_config: ChannelConnConfig,
    ping_interval: Duration,
    valid_pong_duration: Duration,
}

impl Default for WsChannelConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64,
            conn_config: ChannelConnConfig::BindToFirst,
            ping_interval: Duration::from_millis(1000),
            valid_pong_duration: Duration::from_millis(300),
        }
    }
}

pub struct WsChannelInternal {
    channel_listener: TcpListener,
    remote_peer_addr: SocketAddr,
    ws_stream: WebSocketStream<TcpStream>,
    e_sender: Sender<WsChannelConnError>,
    read_sender: Sender<Message>,
    last_ping: Instant,
    last_pong: Instant,
    config: WsChannelConfig,
    send_interval: Interval,
    cancellation_token: CancellationToken,
    send_request_recv: Receiver<Message>,
    mode: WsChannelMode,
    ping_num: u64,
}

impl WsChannelInternal {
    pub async fn new(
        channel_listener: TcpListener,
        e_sender: Sender<WsChannelConnError>,
        read_sender: Sender<Message>,
        config: WsChannelConfig,
        cancellation_token_recv: CancellationToken,
        send_request_recv: Receiver<Message>,
    ) -> Result<Self, Error> {
        info!(target: "comm", "CREATE new WsChannelInternal");

        info!(target: "comm", "CREATE new TcpStream");
        let (tcp_stream, _socket_addr) = channel_listener.accept().await?;

        info!(target: "comm", "CREATED new TcpStream");

        let remote_peer_addr = tcp_stream.peer_addr()?;

        info!(target: "comm", "CREATE new WsStream");
        let ws_stream = accept_async(tcp_stream).await?;

        info!(target: "comm", "CREATED new WsStream");

        let last_pong = Instant::now();
        let last_ping = Instant::now();

        let send_interval = time::interval(config.ping_interval);

        Ok(WsChannelInternal {
            channel_listener,
            remote_peer_addr,
            ws_stream,
            e_sender,
            read_sender,
            last_pong,
            last_ping,
            config,
            send_interval,
            cancellation_token: cancellation_token_recv,
            send_request_recv,
            mode: WsChannelMode::Stable,
            ping_num: 0,
        })
    }

    // Closes the websocket
    pub async fn handle_close(mut self) {
        info!(target: "comm", "HANDLE CLOSE");
        if let Err(e) = self
            .ws_stream
            .close(Some(CloseFrame {
                code: CloseCode::Away,
                reason: "Closed as WsChannel was dropped".into(),
            }))
            .await
        {
            self.e_sender
                .send(e.into())
                .await
                .expect("Error while writing to mpsc channel");
        }
    }

    // Tries sending ping; if there is a connection error: retries
    pub async fn handle_ping(&mut self) {
        info!(target: "comm", "HANDLE PING");
        loop {
            if let Err(e) = self
                .ws_stream
                .send(Message::Ping(Bytes::from_iter(
                    self.ping_num.to_le_bytes().into_iter(),
                )))
                .await
            {
                if let Err(e) = self.handle_recoverable_ws_error(e).await {
                    self.e_sender
                        .send(e)
                        .await
                        .expect("Error while writing to mpsc channel");
                    break;
                }
            } else {
                self.ping_num += 1;
                self.last_ping = Instant::now();
                break;
            }
        }
    }

    // Retries sending the msg, doing reconnect attempts in-between
    pub async fn handle_send(&mut self, msg: Message) {
        info!(target: "comm", "HANDLE SEND {msg:?}");
        loop {
            let send_res = self.ws_stream.send(msg.clone()).await;
            if let Err(e) = send_res {
                if let Err(e) = self.handle_recoverable_ws_error(e).await {
                    self.e_sender
                        .send(e)
                        .await
                        .expect("Error while writing to mpsc channel");
                    break;
                }
            } else {
                break;
            }
        }
    }

    // Tries reconnecting if a recoverable error was read, otherwise processes the message
    pub async fn handle_read(&mut self, read_res: Result<Message, Error>) {
        info!(target: "comm", "HANDLE READ ({read_res:?})");
        match read_res {
            Ok(msg) => match msg {
                Message::Text(_) | Message::Binary(_) => self
                    .read_sender
                    .send(msg)
                    .await
                    .expect("Error while writing to mpsc channel"),
                Message::Pong(_) => {
                    let time_since_last_pong = self.last_pong.elapsed();
                    let time_since_last_ping = self.last_ping.elapsed();

                    self.last_pong = Instant::now();
                    info!(target: "comm", "RECV PONG (latency = {time_since_last_ping:?}, period = {time_since_last_pong:?})");
                }
                Message::Ping(_bytes) => self.handle_ping().await,
                Message::Close(c) => {
                    if let Err(e) = self.reconnect().await {
                        self.e_sender
                            .send(e)
                            .await
                            .expect("Error while writing to mpsc channel");
                    }
                }
                _ => panic!("Unexpected Message type"),
            },
            Err(e) => {
                if let Err(e) = self.handle_recoverable_ws_error(e).await {
                    self.e_sender
                        .send(e)
                        .await
                        .expect("Error while writing to mpsc channel");
                }
            }
        }
    }

    pub async fn handle_recoverable_ws_error(
        &mut self,
        error: Error,
    ) -> Result<(), WsChannelConnError> {
        info!(target: "comm", "HANDLE RECOVERABLE? ATTEMPT");
        match error {
            Error::ConnectionClosed | Error::AlreadyClosed => self.reconnect().await?,
            _ => return Err(error.into()),
        }
        Ok(())
    }

    pub async fn reconnect(&mut self) -> Result<(), WsChannelConnError> {
        if self.config.conn_config == ChannelConnConfig::Once {
            error!(target: "comm", "RECONNECT INVALID --> Channel closed, only open ONCE");
            return Err(ChannelConnError::ChannelClosed.into());
        }
        info!(target: "comm", "RECONNECT searching...");
        let (tcp_stream, new_connection_addr) = self
            .channel_listener
            .accept()
            .await
            .expect("Could not reconnect");

        match self.config.conn_config {
            ChannelConnConfig::Expect(ip_addr) => {
                if new_connection_addr.ip() != ip_addr {
                    error!(target: "comm", "FOUND CONN --> {new_connection_addr} != EXPECT({ip_addr})");
                    return Err(ChannelConnError::RejectedConnection(new_connection_addr).into());
                }
            }
            ChannelConnConfig::BindToFirst => {
                if new_connection_addr.ip() != self.remote_peer_addr.ip() {
                    error!(target: "comm", "FOUND CONN --> {new_connection_addr} != BIND_TO_FIRST({})", self.remote_peer_addr);
                    return Err(ChannelConnError::RejectedConnection(new_connection_addr).into());
                }
            }
            ChannelConnConfig::Once => {
                error!(target: "comm", "FOUND CONN --> Not expected (ONCE)");
                return Err(ChannelConnError::ChannelClosed.into());
            }
            ChannelConnConfig::Any => {}
        }

        info!(target: "comm", "RECONNECT ACCEPTED ({new_connection_addr})");

        // match accept_async(tcp_stream).await {
        //     Ok(ws_stream) => self.ws_stream = ws_stream,
        //     Err(e) => {
        //         self.handle_recoverable_ws_error(e).await?;
        //     }
        // }
        let _ = self
            .ws_stream
            .close(Some(CloseFrame {
                code: CloseCode::Error,
                reason: "ATTEMPTING RECONNECT".into(),
            }))
            .await;
        self.ws_stream = accept_async(tcp_stream).await?;

        Ok(())
    }
}

impl WsChannel {
    pub async fn new(config: WsChannelConfig, port: u16) -> Result<Self, WsChannelConnError> {
        info!(target: "comm", "CREATE new WsChannel (port = {port}, config = {config:?})");

        let listener = TcpListener::bind(("0.0.0.0", port)).await?;

        let (read_sender, read_recv) = tokio::sync::mpsc::channel(config.buffer_size);
        let (send_request_sender,  send_request_recv) =
            tokio::sync::mpsc::channel::<Message>(config.buffer_size);

        let (e_sender, e_recv) = tokio::sync::mpsc::channel(config.buffer_size);

        let cancellation_token = CancellationToken::new();

        let cancellation_token_recv = cancellation_token.clone();

        let mut ws_internal = WsChannelInternal::new(
            listener,
            e_sender,
            read_sender,
            config,
            cancellation_token_recv,
            send_request_recv,
        )
        .await?;

        let ws_external = Self {
            read_recv,
            e_recv,
            send_request_sender,
            cancellation_token,
        };

        tokio::spawn(async move {
            info!(target: "comm", "START WS THREAD");
            loop {
                match ws_internal.mode {
                    // Sending is disabled until the ping indicates, that the connection is stable
                    WsChannelMode::Stabilize => {
                        info!(target: "comm", "STABILIZING!!!");
                        tokio::select! {
                            _ = ws_internal.cancellation_token.cancelled() => {
                                info!(target: "comm", "STABILIZING CANCEL");
                                ws_internal.handle_close().await;
                                break;
                            }
                            _ = ws_internal.send_interval.tick() => {
                                info!(target: "comm", "STABILIZING SEND");
                                ws_internal.handle_ping().await;
                            }
                            read_res = ws_internal.ws_stream.next() => {
                                info!(target: "comm", "STABILIZING READ");
                                if let Some(read_res) = read_res {
                                    ws_internal.handle_read(read_res).await;
                                }
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }

                    // Normal mode --> Can just do all the things
                    WsChannelMode::Stable => {
                        info!(target: "comm", "STABLE TICK");
                        tokio::select! {
                            _ = ws_internal.cancellation_token.cancelled() => {
                                ws_internal.handle_close().await;
                                break;
                            }
                            _ = ws_internal.send_interval.tick() => {
                                ws_internal.handle_ping().await;
                            }
                            read_res = ws_internal.ws_stream.next() => {
                                if let Some(read_res) = read_res {
                                    ws_internal.handle_read(read_res).await;
                                }
                            }
                            send_req = ws_internal.send_request_recv.recv() => {
                                if let Some(send_req) = send_req {
                                    ws_internal.handle_send(send_req).await;
                                }
                            }
                        }
                    }
                }

                let elapsed = ws_internal.last_pong.elapsed();

                if elapsed
                    > ws_internal.config.valid_pong_duration + ws_internal.config.ping_interval
                {
                    info!(target: "comm", "PONG TOO LATE: {elapsed:?}");
                    ws_internal.mode = WsChannelMode::Stabilize;
                } else {
                    ws_internal.mode = WsChannelMode::Stable;
                }
            }
        });

        Ok(ws_external)
    }

    pub async fn read(&mut self) -> Option<Message> {
        self.read_recv.recv().await
    }

    pub async fn next_nonresolved_error(&mut self) -> Option<WsChannelConnError> {
        self.e_recv.recv().await
    }

    pub async fn send(&mut self, msg: Message) {
        self.send_request_sender
            .send(msg)
            .await
            .expect("Error while writing to mpsc channel");
    }
}

impl Drop for WsChannel {
    fn drop(&mut self) {
        info!(target: "comm", "DROPPED WsChannel");
        self.cancellation_token.cancel();
    }
}
