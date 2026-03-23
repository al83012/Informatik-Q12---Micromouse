use std::{thread, time::Duration};

use env_logger::Env;
use futures_util::stream::select;
use log::{info, warn};
use tokio::time;

use crate::comm::{
    heartbeat_channel::{self, HeartbeatWifiChannel, HeartbeatWifiChannelConfig},
    websocket::{self, WsChannelConfig},
    wifi_channel::WifiChannel,
};

pub mod comm;
pub mod direction;
pub mod map;
pub mod measurement;
pub mod position;
pub mod strategy;
pub mod world_data;

#[cfg(test)]
pub mod tests;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("comm=info")).init();

    let mut conn = websocket::WsChannel::new(WsChannelConfig::default(), 9001)
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
