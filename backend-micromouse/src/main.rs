pub mod comm;
pub mod map;
pub mod process;
pub mod strategy;
pub mod transform;
pub mod utils;

#[cfg(test)]
pub mod tests;

#[tokio::main]
async fn main() {
    // init_logging();

    // tests::process_test_short::process_test_short();
    // tracing::info!(target = "main", "STARTUP");
}
