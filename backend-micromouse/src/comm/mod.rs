use std::{io::Error, net::{IpAddr, SocketAddr}, string::FromUtf8Error, sync::Arc, time::Duration};

use futures_util::lock::Mutex;

use crate::comm::wifi_channel::WifiChannel;

pub mod wifi_channel;
pub mod website;
pub mod heartbeat_channel;
pub mod websocket;



#[derive(Debug)]
pub enum ChannelConnError {
    ChannelClosed,
    RejectedConnection(SocketAddr),
    IoError(Error),
    MalformedUtf8(FromUtf8Error),
}



#[derive(Debug, PartialEq)]
pub enum ChannelConnConfig {
    Expect(IpAddr),
    BindToFirst,
    Any,
    Once,
}



impl From<Error> for ChannelConnError {
    fn from(value: Error) -> Self {
        Self::IoError(value)
    }
}

impl From<FromUtf8Error> for ChannelConnError {
    fn from(value: FromUtf8Error) -> Self {
        Self::MalformedUtf8(value)
    }
}
