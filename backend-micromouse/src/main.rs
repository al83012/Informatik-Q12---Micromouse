

use std::time::Duration;

use tokio::time;

use crate::
    comm::esp32::{ WifiChannel}
;

pub mod comm;
pub mod direction;
pub mod map;
pub mod measurement;
pub mod position;

#[cfg(test)]
pub mod tests;

#[tokio::main]
async fn main() {



    let mut channel = WifiChannel::new_on_port(9001, comm::esp32::WifiConnConfig::Any).await;

    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
        res = channel.next_line() => {
            match res {
                Ok(msg) => {
                    println!("Received: {}", msg);
                    break;
                }
                Err(e) => {
                    println!("Read error: {:?}", e);
                    break;
                }
            }
        }

        _ = interval.tick() => {
            if let Err(e) = channel.send("Msg from Laptop").await {
                println!("Write error: {:?}", e);
                break;
            }
        }    }
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
