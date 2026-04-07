use std::{thread, time::Duration};

// use env_logger::{Builder, Env};
use futures_util::stream::select;
use tracing::info;
// use log4rs::{Config, Logger, append::console::ConsoleAppender, config::{Appender, Root}};
use tokio::time;

use crate::{comm::{
    heartbeat_channel::{self, HeartbeatWifiChannel, HeartbeatWifiChannelConfig},
    websocket::{self, WsChannelConfig},
    wifi_channel::WifiChannel,
}, logging::init_logging};

pub mod comm;
pub mod direction;
pub mod map;
pub mod measurement;
pub mod nonempty;
pub mod position;
pub mod stats;
pub mod strategy;
pub mod world_data;
pub mod logging;
pub mod map_display;
pub mod check;

#[cfg(test)]
pub mod tests;

#[tokio::main]
async fn main() {
    init_logging();

    tracing::info!(target = "main", "STARTUP");

    // let stdout = ConsoleAppender::builder().encoder(Box::new()).build();
    //
    // let config = Config::builder()
    //     .appender(Appender::builder().build("stdout", Box::new(stdout)))
    //     .build(Root::builder().appender("stdout")
    //         .build(LevelFilter::Info))
    //     .unwrap();
    //
    // let _handle = log4rs::init_config(config).unwrap();
    // env_logger::Builder::from_env(Env::default().default_filter_or("comm=info")).init();
    // Builder::new()
    //     .format(|buf, record| {
    //         writeln!(
    //             buf,
    //             "{} [{}] - {}",
    //             // Local::now().format("%Y-%m-%dT%H:%M:%S"),
    //             record.level(),
    //             record.args()
    //         )
    //     })
    //     .filter(None, LevelFilter::Info)
    //     .init();

    let conn = websocket::WsChannel::new(WsChannelConfig::default(), 9001)
        .await
        .expect("EXITED WITH ERROR");

    let mut write_tick = time::interval(Duration::from_millis(1500));

    loop {
        tokio::select! {
            e = conn.next_nonresolved_error() => {
                panic!("Unable to resolve error: {e:?}");
            }
            read = conn.read() => {
                info!(target: "comm", "READ (at high level): {read:?}");
            }
            _ = write_tick.tick() => {
                conn.send(tungstenite::Message::Text("AAAAHHHHHHHH".into())).await
            }

        }
    }
}
