use std::{
    io::{Error, ErrorKind},
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

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

pub enum WifiConnConfig {
    Expect(SocketAddr),
    BindToFirst,
    Any,
    Once,
}

impl WifiChannel {
    pub async fn new_on_port(port: u16, conn_config: WifiConnConfig) -> Self {
        let channel_listener = TcpListener::bind(("0.0.0.0", port))
            .await
            .expect("Connection not successful");

        let (tcp_stream, remote_peer_addr) = channel_listener
            .accept()
            .await
            .expect("Could not find peer");

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

    pub async fn reconnect(&mut self) -> Result<(), WifiConnError> {
        let (tcp_stream, new_connection_addr) = self
            .channel_listener
            .accept()
            .await
            .expect("Could not reconnect");

        match self.conn_config {
            WifiConnConfig::Expect(socket_addr) => {
                if new_connection_addr != socket_addr {
                    return Err(WifiConnError::RejectedConnection(new_connection_addr));
                }
            }
            WifiConnConfig::BindToFirst => {
                if new_connection_addr != self.remote_peer_addr {
                    return Err(WifiConnError::RejectedConnection(new_connection_addr));
                }
            }
            WifiConnConfig::Once => return Err(WifiConnError::ChannelClosed),
            WifiConnConfig::Any => {}
        }

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
                Ok(Some(msg)) => return Ok(msg),
                Ok(None) => self.reconnect().await?,
                Err(e) => self.handle_recoverable_io_error(e).await?,
            }
        }
    }

    async fn handle_recoverable_io_error(&mut self, error: Error) -> Result<(), WifiConnError> {
        match error.kind() {
            ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
            | ErrorKind::UnexpectedEof
            | ErrorKind::TimedOut => {
                // Try reconnect
                self.reconnect().await
            }
            _ => Err(WifiConnError::from(error)),
        }
    }

    // Tries to reconnect, if the channel was not allowed to disconnect and did so (even if it was
    // graceful) or if the channel returns a recoverable connection error
    pub async fn send(&mut self, msg: &str) -> Result<usize, WifiConnError> {
        loop {
            let write_res = self.writer.write(msg.as_bytes()).await;

            match write_res {
                Ok(bytes) => return Ok(bytes),
                Err(e) => self.handle_recoverable_io_error(e).await?,
            }
        }
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
