use std::time::Duration;

use env_logger::Env;
use log::{info, warn};
use tokio::time;

use crate::comm::esp32::WifiChannel;

pub mod comm;
pub mod direction;
pub mod map;
pub mod measurement;
pub mod position;

#[cfg(test)]
pub mod tests;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Creating new channel...");
    let mut channel = WifiChannel::new_on_port(9001, comm::esp32::WifiConnConfig::Any).await;
    info!("Connection found: {}", channel.peer_addr());

    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        info!("STEP");
        tokio::select! {
            res = channel.next_line() => {
                info!("STEP --> Next line read");
                match res {
                    Ok(msg) => {
                        info!("Received: {}", msg);
                    }
                    Err(e) => {
                        warn!("Read error: {:?}", e);
                        break;
                    }
                }
            }

            _ = interval.tick() => {

                info!("STEP --> Tried sending message");
                if let Err(e) = channel.send("Msg from Laptop").await {
                    warn!("Write error: {:?}", e);
                    break;
                }
            }
        }
    }

    // channels_to("9001").await;

    // let m = Measurement {
    //     position: Position { x: 0, y: 0 },
    //     direction: Direction::PosY,
    //     value: MeasurementValue::Value { cells: 2 },
    // };
    //
    // let mut map = Map::<4>::new();
    // map.update_discovery(&m).unwrap();
    //

    // println!("{}", map);
}
