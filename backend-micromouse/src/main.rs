use std::time::Duration;

use env_logger::Env;
use log::{info, warn};
use tokio::time;

use crate::comm::wifi_channel::WifiChannel;

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
    info!(target: "prog", "SEARCHING new conn...");
    let mut channel =
        WifiChannel::new_on_port(9001, comm::wifi_channel::WifiConnConfig::BindToFirst).await;
    info!(target: "prog", "FOUND new conn: {}", channel.peer_addr());

    let mut interval = time::interval(Duration::from_millis(250));

    let mut msg = 0;

    const DELIM: u8 = b'$';

    loop {
        tokio::select! {
            res = channel.read_until_delim(DELIM) => {
                info!(target: "comm", "READ");
                match res {
                    Ok(msg) => {
                        info!(target: "comm", "READ OK {msg}");
                    }
                    Err(e) => {
                        warn!("READ ERR: {e:?}");
                        break;
                    }
                }
            }

            _ = interval.tick() => {

                let send_str = format!("message({msg})$");

                info!(target: "comm", "SEND \"{msg}\"");

                if let Err(e) = channel.send(&send_str, DELIM, Duration::from_millis(1000)).await {

                    warn!("SEND ERR: {e:?}");
                    break;
                }
                    msg += 1;
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
