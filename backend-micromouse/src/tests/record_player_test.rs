use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures_util::StreamExt;
use tracing::{error, Instrument};

use crate::{
    tests::frontend_simulator::{self, FrontendSimulator},
    utils::{
        hyperlink_logging::{init_loggers, process_span},
        records::RecordPlayer,
    },
};

#[test]
#[ignore]
fn record_player_test() {
    init_loggers();
    let rt = tokio::runtime::Runtime::new().unwrap();

    // let m_handle = rt.spawn(
    //     async move {
    //         let mut micromouse_simulator = MicromouseSimulator::new(world);
    //         micromouse_simulator.run(3).await;
    //     }
    //     .instrument(process_span("micromouse_sim")),
    // );
    let f_handle = rt.spawn(
        async {
            let mut frontend_simulator = FrontendSimulator::new(Duration::from_hours(200));
            frontend_simulator.run().await;
        }
        .instrument(process_span("frontend_sim")),
    );
    let r_handle = rt.spawn(
        async {
            let mut record_player = RecordPlayer::new()
                .await
                .expect("There has to be a record for this test to run");
            record_player.run().await;
        }
        .instrument(process_span("process")),
    );

    rt.block_on(async {
        tokio::select! {
            // _ = m_handle => {error!(target: "tests", "Micromouse Simulator failed")}
            _ = f_handle => {println!("Frontend failed"); error!(target: "tests", "Frontend Simulator failed")}
            _ = r_handle => {println!("Player failed");error!(target: "tests", "RecordPlayer failed")}
        }
    });
    println!("Finished");
}
