use std::{
    collections::VecDeque,
    io::{Error, ErrorKind},
    net::{IpAddr, SocketAddr},
    str::Utf8Error,
    string::FromUtf8Error,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use futures_util::FutureExt;
use log::{error, info, warn};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Lines},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream, ToSocketAddrs,
    },
};

type WifiWriter = BufWriter<OwnedWriteHalf>;
type WifiReader = BufReader<OwnedReadHalf>;

pub struct WifiChannel {
    channel_listener: TcpListener,
    writer: WifiWriter,
    reader: WifiReader,
    remote_peer_addr: SocketAddr,
    conn_config: WifiConnConfig,
    // read-operation is used for checking whether the connection is still alive -->
    // Send-operations may read single utf8s --> need to be stored and appended to next msg
    read_test_buffer: VecDeque<Result<String, FromUtf8Error>>,
}

#[derive(Debug)]
pub enum WifiConnError {
    ChannelClosed,
    RejectedConnection(SocketAddr),
    IoError(Error),
    MalformedUtf8(FromUtf8Error),
}

impl From<Error> for WifiConnError {
    fn from(value: Error) -> Self {
        Self::IoError(value)
    }
}

impl From<FromUtf8Error> for WifiConnError {
    fn from(value: FromUtf8Error) -> Self {
        Self::MalformedUtf8(value)
    }
}

#[derive(Debug, PartialEq)]
pub enum WifiConnConfig {
    Expect(IpAddr),
    BindToFirst,
    Any,
    Once,
}

impl WifiChannel {
    pub async fn new_on_port(port: u16, conn_config: WifiConnConfig) -> Self {
        info!(target: "comm", "SEARCHING with port = {}, conn_config = {:?}", port, conn_config);
        let channel_listener = TcpListener::bind(("0.0.0.0", port))
            .await
            .expect("Connection not successful");

        info!(target: "comm", "AWAIT first conn");
        let (tcp_stream, remote_peer_addr) = channel_listener
            .accept()
            .await
            .expect("Could not find peer");

        info!(target: "comm", "FOUND first conn");
        let (reader, writer) = tcp_stream.into_split();
        let reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);

        WifiChannel {
            channel_listener,
            reader,
            writer,
            remote_peer_addr,
            conn_config,
            read_test_buffer: VecDeque::new(),
        }
    }

    pub fn peer_addr(&self) -> &SocketAddr {
        &self.remote_peer_addr
    }

    async fn reconnect(&mut self) -> Result<(), WifiConnError> {
        if self.conn_config == WifiConnConfig::Once {
            error!(target: "comm", "RECONNECT INVALID --> Channel closed, only open ONCE");
            return Err(WifiConnError::ChannelClosed);
        }
        info!(target: "comm", "RECONNECT searching...");
        let (tcp_stream, new_connection_addr) = self
            .channel_listener
            .accept()
            .await
            .expect("Could not reconnect");

        match self.conn_config {
            WifiConnConfig::Expect(ip_addr) => {
                if new_connection_addr.ip() != ip_addr {
                    error!(target: "comm", "FOUND CONN --> {new_connection_addr} != EXPECT({ip_addr})");
                    return Err(WifiConnError::RejectedConnection(new_connection_addr));
                }
            }
            WifiConnConfig::BindToFirst => {
                if new_connection_addr.ip() != self.remote_peer_addr.ip() {
                    error!(target: "comm", "FOUND CONN --> {new_connection_addr} != BIND_TO_FIRST({})", self.remote_peer_addr);
                    return Err(WifiConnError::RejectedConnection(new_connection_addr));
                }
            }
            WifiConnConfig::Once => {
                error!(target: "comm", "FOUND CONN --> Not expected (ONCE)");
                return Err(WifiConnError::ChannelClosed);
            }
            WifiConnConfig::Any => {}
        }

        info!(target: "comm", "RECONNECT ACCEPTED ({new_connection_addr})");

        let (reader, writer) = tcp_stream.into_split();
        let reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);

        self.reader = reader;
        self.writer = writer;
        self.remote_peer_addr = new_connection_addr;

        Ok(())
    }

    // Tries to read next line
    // Tries to reconnect, if the channel was not allowed to disconnect and did so (even if it was
    // graceful) or if the channel returns a recoverable connection error
    pub async fn read_until_delim(&mut self, delim: u8) -> Result<String, WifiConnError> {
        if !self.read_test_buffer.is_empty() {
            let fi = self
                .read_test_buffer
                .pop_front()
                .expect("Buffer should not be empty");
            return fi.map_err(|e| e.into());
        }
        loop {
            let mut buf = vec![];
            match self.reader.read_until(delim, &mut buf).await {
                // 0 bytes read --> Connection closed
                Ok(0) => {
                    warn!(target: "comm", "READ 0 bytes --> RECONNECT?");
                    self.reconnect().await?
                }
                // Some positive number of bytes read
                Ok(x) => {
                    info!(target: "comm", "READ {x} bytes");
                    return Ok(String::from_utf8(buf)?);
                }
                Err(e) => {
                    error!(target: "comm", "READ ERROR --> RECONNECT?");
                    self.handle_recoverable_io_error(e).await?
                }
            }
        }
    }

    async fn handle_recoverable_io_error(&mut self, error: Error) -> Result<(), WifiConnError> {
        info!(target: "comm", "HANDLE ERROR? ({error})");
        match error.kind() {
            ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
            | ErrorKind::UnexpectedEof
            | ErrorKind::TimedOut => {
                // Try reconnect

                info!(target: "comm", "RECOVERABLE --> RECONNECT");
                self.reconnect().await
            }
            _ => {
                error!(target: "comm", "NON RECOVERABLE");
                Err(WifiConnError::from(error))
            }
        }
    }

    pub async fn test_read_reconnect(
        &mut self,
        delim: u8,
        error_search_time: Duration,
    ) -> Result<(), WifiConnError> {
        // Checking, whether there is a read available right now
        let mut buf = vec![];
        match tokio::time::timeout(error_search_time, self.reader.read_until(delim, &mut buf)).await {
            Ok(Ok(0)) => {
                warn!(target: "comm", "TEST READ 0 bytes --> RECONNECT?");
                self.reconnect().await?;
            }
            Ok(Ok(x)) => {
                info!(target: "comm", "TEST OK READ {x} bytes --> BUFFER");
                // Added to the message-buffer to be handled later on
                self.read_test_buffer.push_back(String::from_utf8(buf));
            }
            Ok(Err(e)) => {
                error!(target: "comm", "TEST READ ERROR --> RECONNECT?");
                self.handle_recoverable_io_error(e).await?
            }
            Err(e) => {
                info!(target: "comm", "TEST NO SERVER DISCONNECT AFTER {e}");
                // Channel open, no problem immediately obvious
            }
        }

        info!(target: "comm", "TEST current buf = {:?}", self.read_test_buffer);

        return Ok(());
    }

    pub async fn send(&mut self, msg: &str, test_read_delim: u8, error_search_time: Duration) -> Result<(), WifiConnError> {
        info!(target: "comm", "SEND TEST CONN");

        // Trying to read a message at the start to check whether the connection is still alive and
        // maybe reconnecting, while still in the send-portion of the thing
        self.test_read_reconnect(test_read_delim, error_search_time).await?;

        loop {
            let r = self.writer.write_all(msg.as_bytes()).await;
            match r {
                Ok(_) => {
                    break;
                }
                Err(e) => self.handle_recoverable_io_error(e).await?,
            }
        }

        loop {
            let r = self.writer.flush().await;
            match r {
                Ok(_) => {
                    break;
                }
                Err(e) => self.handle_recoverable_io_error(e).await?,
            }
        }

        info!(target: "comm", "SEND SUCCESSFUL");

        Ok(())
    }

    // Tries to reconnect, if the channel was not allowed to disconnect and did so (even if it was
    // graceful) or if the channel returns a recoverable connection error
    async unsafe fn send_maybe_disconnect(&mut self, msg: &str) -> Result<(), WifiConnError> {
        loop {
            let r = self.writer.write_all(msg.as_bytes()).await;
            match r {
                Ok(_) => {
                    break;
                }
                Err(e) => self.handle_recoverable_io_error(e).await?,
            }
        }
        loop {
            let r = self.writer.flush().await;
            match r {
                Ok(_) => {
                    break;
                }
                Err(e) => self.handle_recoverable_io_error(e).await?,
            }
        }
        info!(target: "comm", "SEND MAYBE SUCCESSFUL");

        Ok(())
    }
}

// pub async fn channels_to(first_conn_on_port: &str) {
//     // Host connection
//     let listener = TcpListener::bind(("0.0.0.0", 9001))
//         .await
//         .expect("Connection not successful");
//
//     // loop {
//     // Get stream to first thing that connects to channel
//     let (tcp_stream, _) = listener.accept().await.unwrap();
//     let (mut read_stream, mut write_stream) = tcp_stream.into_split();
//
//     // Spawn reader
//     tokio::spawn(async move {
//         println!("Reader thread");
//         let reader = BufReader::new(read_stream);
//         let mut lines = reader.lines();
//         while let Some(line) = lines.next_line().await.unwrap() {
//             println!("Received: {}", line);
//         }
//         println!("Connection --> Closed by other");
//     });
//
//     // Writer loop for this connection
//
//     println!("Writer thread");
//     loop {
//         if let Err(e) = write_stream.write_all(b"asdjfh\n").await {
//             println!("Failed to write: {}", e);
//             break;
//         }
//         tokio::time::sleep(Duration::from_secs(1)).await;
//     }
//     // }
// }
