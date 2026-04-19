
// use env_logger::{Builder, Env};
// use log4rs::{Config, Logger, append::console::ConsoleAppender, config::{Appender, Root}};


use crate::utils::logging::init_logging;

pub mod comm;
pub mod map;
pub mod utils;
pub mod strategy;
pub mod transform;
pub mod process;


#[cfg(test)]
pub mod tests;

#[tokio::main]
async fn main() {
    init_logging();

    tracing::info!(target = "main", "STARTUP");

    
}
