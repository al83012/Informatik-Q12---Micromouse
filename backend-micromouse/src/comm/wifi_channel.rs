use std::{
    io::{Error, ErrorKind},
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use log::info;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Lines},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream, ToSocketAddrs,
    },
};

type WifiWriter = BufWriter<OwnedWriteHalf>;
type WifiReader = Lines<BufReader<OwnedReadHalf>>;

pub struct WifiChannel {
    channel_listener: TcpListener,
    writer: WifiWriter,
    reader: WifiReader,
    remote_peer_addr: SocketAddr,
    conn_config: WifiConnConfig,
}

#[derive(Debug)]
pub enum WifiConnError {
    ChannelClosed,
    RejectedConnection(SocketAddr),
    IoError(Error),
}

impl From<Error> for WifiConnError {
    fn from(value: Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Debug)]
pub enum WifiConnConfig {
    Expect(SocketAddr),
    BindToFirst,
    Any,
    Once,
}

impl WifiChannel {
    pub async fn new_on_port(port: u16, conn_config: WifiConnConfig) -> Self {
        info!("port = {}, conn_config = {:?}", port, conn_config);
        let channel_listener = TcpListener::bind(("0.0.0.0", port))
            .await
            .expect("Connection not successful");

        info!("    awaiting connection...");
        let (tcp_stream, remote_peer_addr) = channel_listener
            .accept()
            .await
            .expect("Could not find peer");

        info!("    found connection");
        let (reader, writer) = tcp_stream.into_split();
        let reader = BufReader::new(reader).lines();
        let writer = BufWriter::new(writer);

        WifiChannel {
            channel_listener,
            reader,
            writer,
            remote_peer_addr,
            conn_config,
        }
    }

    pub fn peer_addr(&self) -> &SocketAddr {
        &self.remote_peer_addr
    }

    pub async fn reconnect(&mut self) -> Result<(), WifiConnError> {
        info!("Attempting to reconnect: Awaiting connection");
        let (tcp_stream, new_connection_addr) = self
            .channel_listener
            .accept()
            .await
            .expect("Could not reconnect");

        match self.conn_config {
            WifiConnConfig::Expect(socket_addr) => {
                if new_connection_addr != socket_addr {
                    info!("Found new connection ({}), which does not match the expected connection ({})", new_connection_addr, socket_addr);
                    return Err(WifiConnError::RejectedConnection(new_connection_addr));
                }
            }
            WifiConnConfig::BindToFirst => {
                if new_connection_addr != self.remote_peer_addr {
                    info!("Found new connection ({}), which does not match the previous connection ({})", new_connection_addr, self.remote_peer_addr);
                    return Err(WifiConnError::RejectedConnection(new_connection_addr));
                }
            }
            WifiConnConfig::Once => {
                info!("Reconnection aborted: Was set to only connect once");
                return Err(WifiConnError::ChannelClosed);
            }
            WifiConnConfig::Any => {}
        }

        info!("Accepted reconnect: {}", new_connection_addr);

        let (reader, writer) = tcp_stream.into_split();
        let reader = BufReader::new(reader).lines();
        let writer = BufWriter::new(writer);

        self.reader = reader;
        self.writer = writer;
        self.remote_peer_addr = new_connection_addr;

        Ok(())
    }

    // Tries to read next line
    // Tries to reconnect, if the channel was not allowed to disconnect and did so (even if it was
    // graceful) or if the channel returns a recoverable connection error
    pub async fn next_line(&mut self) -> Result<String, WifiConnError> {
        loop {
            match self.reader.next_line().await {
                Ok(Some(msg)) => {
                    info!("NL --> return msg");
                    return Ok(msg);
                }
                Ok(None) => {
                    info!("NL --> Connection closed with EOF --> check, whether disconnect was permitted");
                    self.reconnect().await?
                }
                Err(e) => {
                    info!("Connection error: {e} --> Might be recoverable");
                    self.handle_recoverable_io_error(e).await?
                }
            }
        }
    }

    async fn handle_recoverable_io_error(&mut self, error: Error) -> Result<(), WifiConnError> {
        info!("Test for recoverable error");
        match error.kind() {
            ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
            | ErrorKind::UnexpectedEof
            | ErrorKind::TimedOut => {
                // Try reconnect

                info!("Error recoverable --> try reconnect");
                self.reconnect().await
            }
            _ => {
                info!("Non-recoverable io-Error: {error}");
                Err(WifiConnError::from(error))
            }
        }
    }

    // Tries to reconnect, if the channel was not allowed to disconnect and did so (even if it was
    // graceful) or if the channel returns a recoverable connection error
    pub async fn send(&mut self, msg: &str) -> Result<(), WifiConnError> {
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
            let r = self.writer.write_all(b"\n").await;

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
