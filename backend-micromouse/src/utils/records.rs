use std::{
    fs::{self, File, FileType},
    io::{BufRead, BufReader, BufWriter, Seek, Write},
    path::PathBuf,
};

use chrono::Local;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use futures_util::{Stream, StreamExt};
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::info;

use crate::{
    comm::{
        website::{BatchedFrontendMessage, FrontendMessage},
        websocket::{WsChannel, WsChannelConfig},
    },
    process::Process,
};

pub struct RecordWriter {
    writer: File,
    init: bool,
}

impl RecordWriter {
    pub fn new() -> std::io::Result<Self> {
        let record_folder_path = PathBuf::from("records");
        fs::create_dir_all(&record_folder_path)?;

        let record_id = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

        let mut writer =
            fs::File::create(record_folder_path.join(record_id).with_extension("json"))?;

        writer.write_all("[\n\n]".as_bytes())?;

        Ok(Self { writer, init: true })
    }

    pub fn write(&mut self, frontend_message: &BatchedFrontendMessage) -> std::io::Result<()> {
        self.writer.seek(std::io::SeekFrom::End(-2))?;

        if !self.init {
            self.writer.write_all(b",\n")?;
        } else {
            self.init = false;
        }

        serde_json::to_writer(&mut self.writer, frontend_message)?;

        self.writer.write_all(b"\n]")?;
        self.writer.flush()?;

        Ok(())
    }
}

impl Drop for RecordWriter {
    fn drop(&mut self) {
        self.writer.flush().expect("Panic at last flush")
    }
}

pub struct RecordPlayer {
    channel: WsChannel,
    reader: BufReader<File>,
    current_batch: Option<BatchedFrontendMessage>,
}

impl RecordPlayer {
    pub async fn new() -> Option<Self> {
        info!(target: "rec", "Creating RecordPlayer");
        let record_folder_path = PathBuf::from("records");
        let mut files = fs::read_dir(&record_folder_path)
            .ok()?
            .into_iter()
            .filter_map(|e| {
                let Ok(e) = e else {
                    return None;
                };
                if e.file_name().to_string_lossy().ends_with(".json") {
                    Some(e)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        files.sort_by_key(|f| f.file_name());

        let last = files.last()?;

        let path = last.path();
        info!(target: "rec", "Opening {path:?}");
        let mut reader = BufReader::new(File::open(path).ok()?);

        // skip first line
        reader.read_line(&mut String::new());

        let channel = WsChannel::new(WsChannelConfig::default(), Process::<0>::FRONTEND_PORT)
            .await
            .ok()?;

        Some(Self {
            reader,
            current_batch: None,
            channel,
        })
    }
    pub fn next_msg(&mut self) -> Option<FrontendMessage> {
        if let Some(BatchedFrontendMessage(ref mut current)) = self.current_batch {
            if !current.is_empty() {
                return Some(current.remove(0));
            } else {
                self.current_batch = None;
            }
        }
        self.current_batch = self.next_batch();
        if self.current_batch.is_some() {
            self.next_msg()
        } else {
            None
        }
    }
    pub fn next_batch(&mut self) -> Option<BatchedFrontendMessage> {
        if self.current_batch.is_some() {
            return self.current_batch.take();
        }

        let mut next_line = String::new();
        let _read_count = self.reader.read_line(&mut next_line).ok()?;

        let next_line = next_line.trim().trim_end_matches(",");
        // if next_line == "]" {
        //     return None;
        // }

        serde_json::from_str(next_line).ok()
    }

    pub async fn run(&mut self) {
        let (event_send, mut event_recv) = tokio::sync::mpsc::unbounded_channel();

        let mut event_stream = EventStream::new();

        tokio::spawn(async move {
            while let Some(Ok(event)) = event_stream.next().await {
                info!(target: "rec", "Sending event");
                event_send.send(event).expect("Failed to send via channel");
            }
        });

        while let Some(next) = self.next_requested(&mut event_recv).await {
            info!(target: "rec", "Sending {next:#?}");
            self.channel.send(next.into()).await;
            info!(target: "rec", "Awaiting next signal");
        }

        info!(target: "rec", "Closing record player");
    }

    pub async fn next_requested(
        &mut self,
        event_recv: &mut UnboundedReceiver<Event>,
    ) -> Option<BatchedFrontendMessage> {
        info!(target: "rec", "Selecting next signal");
        loop {
            info!(target: "rec", "Awaiting event");
            let next_event = event_recv.recv().await?;

            info!(target: "rec", "Received event");

            if let Event::Key(key_event) = next_event {
                if key_event.code != KeyCode::Right {
                    continue;
                }
                if key_event.modifiers == KeyModifiers::NONE {
                    info!(target: "rec", "Do single batch");
                    return Some(BatchedFrontendMessage(vec![self.next_msg()?]));
                } else if key_event.modifiers == KeyModifiers::SHIFT {
                    info!(target: "rec", "Do multi batch");
                    return self.next_batch();
                }
            }
        }
    }
}
