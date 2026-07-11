use tracing::Instrument;

use crate::utils::{hyperlink_logging::{init_loggers, process_span}, records::RecordPlayer};

#[test]
#[ignore]
fn run_record_player() {
    init_loggers();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(
        async {
            let mut record_player = RecordPlayer::new()
                .await
                .expect("There has to be a record for this test to run");
            record_player.run().await;
        }
        .instrument(process_span("record_player")),
    );
}
