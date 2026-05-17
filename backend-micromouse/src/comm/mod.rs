use std::{
    io::Error,
    net::{IpAddr, SocketAddr},
    string::FromUtf8Error,
};

use serde::Serialize;

pub mod micromouse_manager;
pub mod micromouse_message;
pub mod website;
pub mod websocket;

#[deprecated = "Use the websocket channel, it is more reliable"]
pub mod wifi_channel;

#[deprecated = "Use the websocket channel, it is more reliable and already includes a heartbeat/ping"]
pub mod heartbeat_channel;

#[derive(Debug, Serialize)]
pub enum ChannelConnError {
    ChannelClosed,
    RejectedConnection(SocketAddr),
    IoError(#[serde(skip)] Error),
    MalformedUtf8(#[serde(skip)] FromUtf8Error),
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
