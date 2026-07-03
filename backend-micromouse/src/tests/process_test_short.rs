use std::time::Duration;

use tracing::{error, info, span, Instrument};

use crate::{
    process::Process,
    tests::{
        frontend_simulator::{self, FrontendSimulator},
        micromouse_simulator::{self, MicromouseSimulator},
    },
    utils::{
        hyperlink_logging::{init_tree_logger, process_span},
        logging::init_logging,
    },
};

#[test]
#[ignore]
pub fn process_test_short() {
    const N: usize = super::TEST_MAP_SIZE;
    let world = super::test_map(0.5);
    init_tree_logger();
    // let _g = init_logging();
    info!(target: "tests", "Fully shorted process test");
    let rt = tokio::runtime::Runtime::new().unwrap();

    let m_handle = rt.spawn(
        async move {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let mut micromouse_simulator = MicromouseSimulator::new(world, Duration::from_millis(100));
            micromouse_simulator.run(3).await;
        }
        .instrument(process_span("micromouse_sim")),
    );
    let f_handle = rt.spawn(
        async {
            let mut frontend_simulator = FrontendSimulator::new(Duration::from_secs(20));
            frontend_simulator.run().await;
        }
        .instrument(process_span("frontend_sim")),
    );
    let p_handle = rt.spawn(
        async {
            let process: Process<N> = Process::new().await.expect("Process creation failed");
            process.run().await
        }
        .instrument(process_span("process")),
    );

    rt.block_on(async {
        tokio::select! {
            _ = m_handle => {error!(target: "tests", "Micromouse Simulator failed")}
            _ = f_handle => {error!(target: "tests", "Frontend Simulator failed")}
            _ = p_handle => {error!(target: "tests", "Process failed")}
        }
    });
}
