use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

use tracing::Subscriber;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::{fmt, Layer};

use crate::utils::logging::MyFormatter;

pub fn file_appender(target: &str, name: &str) -> (NonBlocking, WorkerGuard) {
    let file_appender = rolling::daily(".", format!("TARGET_{name}.log"));
    tracing_appender::non_blocking(file_appender)
}

