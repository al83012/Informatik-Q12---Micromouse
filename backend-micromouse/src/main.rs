use std::{thread, time::Duration};

use env_logger::Env;
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

#[cfg(test)]
pub mod tests;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("comm=info")).init();

    let conn = websocket::WsChannel::new(WsChannelConfig::default(), 9001).await.expect("EXITED WITH ERROR");

    thread::sleep(Duration::from_secs(60));
    

}
