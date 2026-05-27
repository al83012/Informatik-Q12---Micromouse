use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, span, warn, Instrument, Level};
use tungstenite::{Message, Utf8Bytes};

use crate::{
    comm::micromouse_message::{
        CommandMessage, InterruptAction, InterruptOccurence, MeasurementMessage,
        MeasurementOccurrence, MicromouseResponse, TransformedMovement,
    },
    map::{
        map::Map,
        measurement::{self, MeasurementValue},
        world_data::{self, WorldData},
    },
    transform::position::MouseTransform,
    utils::hyperlink_logging::{enter_process, process_span, LinkFileName},
};

pub struct FrontendSimulator;

impl FrontendSimulator {
    #[instrument(name = "new FrontendSimulator")]
    pub fn new() -> Self {
        Self {}
    }
    #[instrument(skip(self), name = "run")]
    pub async fn run(&mut self) {
        let (mut ws_stream, response) = tokio_tungstenite::connect_async("ws://localhost:9002")
            .await
            .expect("Connection failed");
        info!(target: "test/sim", " < Connection Response = {response:?}");

        ws_stream
            .send(Message::Text(Utf8Bytes::from("RESTART".to_string())))
            .await
            .expect("Error sending opening msg");
        // ws_stream
        //     .send(Message::Text(Utf8Bytes::from("CONTINUE".to_string())))
        //     .await
        //     .expect("Error sending opening msg");

    }
}
