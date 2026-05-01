use std::{io::ErrorKind, net::SocketAddr, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
    time::{self, Instant, Interval},
};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_util::{bytes::Buf, sync::CancellationToken};
use tracing::{debug, error, info, instrument, warn};
use tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    Error, Message,
};

use crate::comm::{ChannelConnConfig, ChannelConnError};
#[cfg(feature = "comm_stats")]
use crate::utils::stats::StatAccumulator;

pub struct WsChannel {
    // There is no specific read-request, as we are reading continuously
    read_recv: Mutex<Receiver<Message>>,
    e_recv: Mutex<Receiver<WsChannelConnError>>,
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
    nodelay: bool,
}

impl Default for WsChannelConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64,
            conn_config: ChannelConnConfig::BindToFirst,
            ping_interval: Duration::from_millis(1000),
            valid_pong_duration: Duration::from_millis(300),
            nodelay: false,
        }
    }
}

pub struct WsChannelInternal {
    remote_peer_addr: SocketAddr,
    ws_stream: Option<WebSocketStream<TcpStream>>,
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
    port: u16,
    #[cfg(feature = "comm_stats")]
    latency_stats: StatAccumulator,
    // latency_stats: incr_stats::incr::Stats,
    #[cfg(feature = "comm_stats")]
    period_stats: StatAccumulator,
    // period_stats: incr_stats::incr::Stats,
}

impl WsChannelInternal {
    #[instrument(
        name = "new WsChannelInternal",
        fields(description = "Create new internal for Ws-Channel running on separate thread")
    )]
    pub async fn new(
        channel_listener: TcpListener,
        e_sender: Sender<WsChannelConnError>,
        read_sender: Sender<Message>,
        config: WsChannelConfig,
        cancellation_token_recv: CancellationToken,
        send_request_recv: Receiver<Message>,
        port: u16,
    ) -> Result<Self, Error> {
        info!(target: "comm", "CREATE new WsChannelInternal");

        info!(target: "comm", "CREATE new TcpStream");
        let (tcp_stream, _socket_addr) = channel_listener.accept().await?;

        tcp_stream.set_nodelay(config.nodelay)?;

        info!(target: "comm", "CREATED new TcpStream");

        let remote_peer_addr = tcp_stream.peer_addr()?;

        info!(target: "comm", "CREATE new WsStream");
        let ws_stream = Some(accept_async(tcp_stream).await?);

        info!(target: "comm", "CREATED new WsStream");

        let last_pong = Instant::now();
        let last_ping = Instant::now();

        let send_interval = time::interval(config.ping_interval);

        Ok(WsChannelInternal {
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
            port,
            #[cfg(feature = "comm_stats")]
            latency_stats: StatAccumulator::new(1000),
            // latency_stats: incr_stats::incr::Stats::new(),
            #[cfg(feature = "comm_stats")]
            period_stats: StatAccumulator::new(1000),
            // period_stats: incr_stats::incr::Stats::new(),
        })
    }

    // Closes the websocket
    #[instrument(
        name = "handle_close",
        fields(description = "Closes the websocket"),
        skip(self)
    )]
    pub async fn handle_close(self) {
        let ws_stream = self.ws_stream;
        if ws_stream.is_none() {
            return;
        }
        let mut ws_stream = ws_stream.unwrap();
        info!(target: "comm", "HANDLE CLOSE");
        if let Err(e) = ws_stream
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
    #[instrument(
        name = "handle_ping",
        fields(description = "Check whether ping was too late (and maybe attempt recovery)"),
        skip(self)
    )]
    pub async fn handle_ping(&mut self) {
        debug!(target: "comm", "HANDLE PING ({})", self.ping_num);
        if let Err(e) = self
            .ws_stream
            .as_mut()
            .expect("WS Stream should exist outside reconnects")
            .send(Message::Ping(self.ping_num.to_string().into_bytes().into()))
            .await
        {
            if let Err(e) = self.handle_recoverable_ws_error(e).await {
                self.e_sender
                    .send(e)
                    .await
                    .expect("Error while writing to mpsc channel");
            }
        } else {
            self.ping_num += 1;
            self.last_ping = Instant::now();
        }
    }

    // Retries sending the msg, doing reconnect attempts in-between
    #[instrument(
        name = "handle_send",
        fields(description = "Sends the message (and maybe attempts recovery)"),
        skip(self)
    )]
    pub async fn handle_send(&mut self, msg: Message) {
        debug!(target: "comm", "HANDLE SEND {msg:?}");
        loop {
            let send_res = self
                .ws_stream
                .as_mut()
                .expect("WS Stream should exist outside reconnects")
                .send(msg.clone())
                .await;
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
    #[instrument(
        name = "handle_read",
        fields(description = "Parse message and distribute to other handle-methods"),
        skip(self)
    )]
    pub async fn handle_read(&mut self, read_res: Result<Message, Error>) {
        debug!(target: "comm", "HANDLE READ ({read_res:?})");
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

                    if time_since_last_ping.as_millis() < 50 {
                        debug!(target: "comm", "RECV PONG (latency = {time_since_last_ping:?}, period = {time_since_last_pong:?})");
                    } else {
                        warn!(target: "comm", "RECV PONG (latency = {time_since_last_ping:?}, period = {time_since_last_pong:?})");
                    }
                    #[cfg(feature = "comm_stats")]
                    {
                        self.latency_stats
                            .add(time_since_last_ping.as_millis() as f64);
                        self.period_stats
                            .add(time_since_last_pong.as_millis() as f64);

                        let stat_count = self.ping_num;
                        let avg_latency = self.latency_stats.avg().unwrap_or(0.0);
                        let avg_period = self.period_stats.avg().unwrap_or(0.0);

                        debug!(target: "comm/stats", "AVG LATENCY: Ø = {avg_latency:.0}");
                        debug!(target: "comm/stats", "AVG PERIOD : Ø = {avg_period:.0}");

                        if stat_count.is_multiple_of(50) && stat_count >= 50 {
                            let sd_latency = self.latency_stats.standard_deviation().unwrap_or(0.0);
                            let sd_period = self.period_stats.standard_deviation().unwrap_or(0.0);

                            debug!(target: "comm/stats", "SD LATENCY: σ = {sd_latency:.0}");
                            debug!(target: "comm/stats", "SD PERIOD : σ = {sd_period:.0}");

                            const NUM_OF_CHUNKS: usize = 10;

                            let percentiles_latency =
                                self.latency_stats.percentile_chunks(NUM_OF_CHUNKS);
                            let percentiles_period =
                                self.period_stats.percentile_chunks(NUM_OF_CHUNKS);

                            if let Some(percentiles_latency) = percentiles_latency {
                                debug!(target: "comm/stats", "{}", percentiles_latency);
                            }
                            if let Some(percentiles_period) = percentiles_period {
                                debug!(target: "comm/stats", "{}", percentiles_period);
                            }
                        }
                        // let _ = self.latency_stats.array_update(&[time_since_last_ping.as_millis() as f64]);
                        // let _ = self.period_stats.array_update(&[time_since_last_pong.as_millis() as f64]);
                        // let avg_latency = self.latency_stats.sum().unwrap() as u32 / self.latency_stats.count();
                        // let avg_period = self.period_stats.sum().unwrap() as u32 / self.latency_stats.count();
                        // let sd_latency = self.latency_stats.sample_standard_deviation().map(|v| v.to_string()).unwrap_or("/".to_string());
                        // let sd_period = self.period_stats.sample_standard_deviation().map(|v| v.to_string()).unwrap_or("/".to_string());
                        // info!(target: "comm", "LATENCY: Ø = {avg_latency}, σ = {sd_latency}");
                        // info!(target: "comm", "PERIOD : Ø = {avg_period}, σ = {sd_period}");
                    }
                    self.last_pong = Instant::now();
                }
                Message::Ping(_bytes) => self.handle_ping().await,
                Message::Close(_c) => {
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

    #[instrument(
        name = "handle_recoverable_ws_error",
        fields(
            description = "Check whether the tungstenite error is recoverable and maybe attempt recovery"
        ),
        skip(self)
    )]
    pub async fn handle_recoverable_ws_error(
        &mut self,
        error: Error,
    ) -> Result<(), WsChannelConnError> {
        info!(target: "comm", "HANDLE RECOVERABLE? ATTEMPT");
        match error {
            Error::ConnectionClosed | Error::AlreadyClosed => {
                warn!(target: "comm", "WS CLOSED --> RECONNECT?");
                self.reconnect().await?
            }
            Error::Io(io_err) => {
                warn!(target: "comm", "IO ERROR");
                self.handle_recoverable_io_error(io_err).await?;
            }
            _ => {
                error!(target: "comm", "NON RECOVERABLE WS ERROR {error:?}");
            }
        }
        Ok(())
    }

    #[instrument(
        name = "handle_recoverable_io_error",
        fields(description = "Check whether io error is recoverable and maybe attempt recovery"),
        skip(self)
    )]
    async fn handle_recoverable_io_error(
        &mut self,
        error: std::io::Error,
    ) -> Result<(), WsChannelConnError> {
        debug!(target: "comm", "HANDLE ERROR? ({error})");
        match error.kind() {
            ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
            | ErrorKind::UnexpectedEof
            | ErrorKind::TimedOut => {
                // Try reconnect

                debug!(target: "comm", "RECOVERABLE --> RECONNECT");
                self.reconnect().await
            }
            _ => {
                error!(target: "comm", "NON RECOVERABLE");
                Err(WsChannelConnError::ChannelConnError(error.into()))
            }
        }
    }

    #[instrument(
        name = "reconnect",
        fields(description = "Try to reconnect (potentially with another device)"),
        skip(self)
    )]
    pub async fn reconnect(&mut self) -> Result<(), WsChannelConnError> {
        if self.config.conn_config == ChannelConnConfig::Once {
            error!(target: "comm", "RECONNECT INVALID --> Channel closed, only open ONCE");
            return Err(ChannelConnError::ChannelClosed.into());
        }
        info!(target: "comm", "RECONNECT searching...");
        let new_listener = TcpListener::bind(("0.0.0.0", self.port)).await?;
        let (tcp_stream, new_connection_addr) =
            new_listener.accept().await.expect("Could not reconnect");
        tcp_stream.set_nodelay(self.config.nodelay)?;

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

        info!(target: "comm", "CLOSING OLD WS");
        let _ = self
            .ws_stream
            .as_mut()
            .expect("Even in large parts of the reconnect, ws should exist")
            .close(Some(CloseFrame {
                code: CloseCode::Error,
                reason: "ATTEMPTING RECONNECT".into(),
            }))
            .await;

        drop(self.ws_stream.take());

        info!(target: "comm", "OPENING NEW WS");
        self.last_ping = Instant::now();
        self.last_pong = Instant::now();
        let res = accept_async(tcp_stream).await;
        info!(target: "comm", "NEW WS = {res:?}");
        self.ws_stream = Some(res?);

        info!(target: "comm", "FINISHED RECONNECT");

        Ok(())
    }
}

impl WsChannel {
    #[instrument(
        name = "new WsChannel",
        fields(description = "Create new user-side struct for the WsChannel")
    )]
    pub async fn new(config: WsChannelConfig, port: u16) -> Result<Self, WsChannelConnError> {
        info!(target: "comm", "CREATE new WsChannel (port = {port}, config = {config:?})");

        let listener = TcpListener::bind(("0.0.0.0", port)).await?;

        info!(target: "comm", "CREATED new TcpListener");

        let (read_sender, read_recv) = tokio::sync::mpsc::channel(config.buffer_size);
        let (send_request_sender, send_request_recv) =
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
            port,
        )
        .await?;

        let ws_external = Self {
            read_recv: Mutex::from(read_recv),
            e_recv: Mutex::from(e_recv),
            send_request_sender,
            cancellation_token,
        };

        tokio::spawn(async move {
            info!(target: "comm", "START WS THREAD");
            loop {
                let stabilize_in_or_reconnect =
                    tokio::time::sleep_until(ws_internal.last_pong + Duration::from_secs(10));

                match ws_internal.mode {
                    // Sending is disabled until the ping indicates, that the connection is stable
                    WsChannelMode::Stabilize => {
                        warn!(target: "comm", "STABILIZING!!!");
                        tokio::select! {
                            _ = ws_internal.cancellation_token.cancelled() => {
                                debug!(target: "comm", "STABILIZING CANCEL");
                                ws_internal.handle_close().await;
                                break;
                            }
                            _ = ws_internal.send_interval.tick() => {
                                debug!(target: "comm", "STABILIZING PING");
                                ws_internal.handle_ping().await;
                            }
                            read_res = ws_internal.ws_stream.as_mut().expect("WS Stream should be Some outside reconnect").next() => {
                                debug!(target: "comm", "STABILIZING READ");
                                if let Some(read_res) = read_res {
                                    debug!(target: "comm", "READ SOME");
                                    ws_internal.handle_read(read_res).await;
                                } else {
                                    let res = ws_internal.handle_recoverable_ws_error(Error::ConnectionClosed).await;
                                    if let Err(res) = res {
                                        error!(target: "comm", "Reopening failed");
                                    }

                                    debug!(target: "comm", "READ NONE");
                                }
                            }
                            _ = stabilize_in_or_reconnect => {
                                error!(target: "comm", "Stabilizing took too long; reopening connection");
                                let res = ws_internal.handle_recoverable_ws_error(Error::ConnectionClosed).await;
                                if let Err(res) = res {
                                    error!(target: "comm", "Reopening failed");
                                }
                            }
                        }
                    }

                    // Normal mode --> Can just do all the things
                    WsChannelMode::Stable => {
                        info!(target: "comm", "STABLE TICK");
                        tokio::select! {
                            _ = ws_internal.cancellation_token.cancelled() => {
                                debug!(target: "comm", "STABLE CANCEL");
                                ws_internal.handle_close().await;
                                break;
                            }
                            _ = ws_internal.send_interval.tick() => {
                                debug!(target: "comm", "STABLE PING");
                                ws_internal.handle_ping().await;
                            }
                            read_res = ws_internal.ws_stream.as_mut().expect("WS Stream should be Some outside reconnect").next() => {
                                debug!(target: "comm", "STABLE READ");
                                if let Some(read_res) = read_res {
                                    ws_internal.handle_read(read_res).await;
                                }
                            }
                            send_req = ws_internal.send_request_recv.recv() => {
                                debug!(target: "comm", "STABLE SEND");
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
                    error!(target: "comm", "PONG TOO LATE: {elapsed:?}");
                    ws_internal.mode = WsChannelMode::Stabilize;
                } else {
                    ws_internal.mode = WsChannelMode::Stable;
                }
            }
        });

        Ok(ws_external)
    }

    /// Returns Some(msg) if there is a message and false if the Channel has already been dropped
    #[instrument(name = "read", skip(self), fields(description = "Read next message"))]
    pub async fn read(&self) -> Option<Message> {
        self.read_recv.lock().await.recv().await
    }

    #[instrument(name = "next_nonresolved_error", skip(self), fields(description = "Like read, but only records the errors"))]
    pub async fn next_nonresolved_error(&self) -> Option<WsChannelConnError> {
        self.e_recv.lock().await.recv().await
    }

    #[instrument(name = "send", skip(self), fields(description = "Send message"))]
    pub async fn send(&self, msg: Message) {
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
