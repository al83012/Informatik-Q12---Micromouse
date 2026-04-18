
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling;


pub fn file_appender(_target: &str, name: &str) -> (NonBlocking, WorkerGuard) {
    let file_appender = rolling::daily(".", format!("TARGET_{name}.log"));
    tracing_appender::non_blocking(file_appender)
}

