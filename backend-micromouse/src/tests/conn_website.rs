use tracing::info;

use crate::{
    comm::website::{FrontendConnectionConfig, FrontendManager, FrontendMessage},
    utils::logging::init_logging,
};

#[test]
#[ignore]
fn test_simple_conn() {
    let guards = init_logging();
    let rt = tokio::runtime::Runtime::new().unwrap();
    info!(target: "test/webs", "Attempting simple connection");
    rt.block_on(async {
        info!(target: "test/webs", "Pre Manager Creation");
        let mut frontend_manager = FrontendManager::new(8090, FrontendConnectionConfig::default())
            .await
            .expect("Error on create");
        info!(target: "test/webs", "Post Manager Creation");

        loop {
            let read = frontend_manager.next_read().await;
            info!(target: "test/webs", "Read {read:?}");
        }
    });
}
