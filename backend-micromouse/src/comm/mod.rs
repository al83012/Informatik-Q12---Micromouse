use std::{sync::Arc, time::Duration};

use futures_util::lock::Mutex;

use crate::comm::wifi_channel::WifiChannel;

pub mod wifi_channel;
pub mod website;
pub mod heartbeat_channel;


