use tracing::instrument;

use crate::{
    comm::{
        micromouse_manager::{self, MicromouseEvent, MicromouseManager, MicromouseManagerError},
        website::{FrontendManager, FrontendMessage, FrontendResponse},
    },
    strategy::{strategy::Strategy, strategy_tree::StrategyTree},
};

pub struct Process<const N: usize> {
    micromouse_manager: MicromouseManager<N>,
    frontend_manager: FrontendManager<N>,
    // strategy_tree: DynamicStrategyTree<N>
}

impl<const N: usize> Process<N> {
    #[instrument(name = "new Process", fields(description = "Create the main process"))]
    pub async fn new() -> Self {
        todo!("Create the connections")
    }

    #[instrument(
        skip_all,
        name = "run",
        fields(description = "Execute the main process")
    )]
    pub async fn run(mut self) {
        loop {
            let micromouse_response = self.micromouse_manager.await_next_read();
            let frontend_response = self.frontend_manager.next_read();
            tokio::select! {
                micromouse_msg = micromouse_response => {
                    let micromouse_event = self.micromouse_manager.process_next_read(micromouse_msg).await;
                    match micromouse_event {
                        Ok(events) => {
                            for event in events {
                                self.handle_micromouse_event(event).await;
                            }
                        }
                        Err(e) => {
                            self.handle_micromouse_error(e).await
                        }
                    }
                }
                frontend_msg = frontend_response => {
                    self.handle_frontend_command(frontend_msg).await;
                }
            }
        }
    }

    #[instrument(
        skip(self),
        name = "handle_micromouse_error",
        fields(description = "Handle micromouse error and maybe propagate it")
    )]
    pub async fn handle_micromouse_error(&mut self, micromouse_error: MicromouseManagerError) {
        self.frontend_manager
            .send(FrontendMessage::MicromouseEvent(MicromouseEvent::Error(
                micromouse_error,
            )))
            .await;
        todo!("Send the error to the frontend and maybe handle it internally");
    }

    #[instrument(
        skip(self),
        name = "handle_micromouse_event",
        fields(
            description = "Handle micromouse event, apply it to the strategy tree and propagate it"
        )
    )]
    pub async fn handle_micromouse_event(&mut self, micromouse_event: MicromouseEvent) {
        self.frontend_manager
            .send(FrontendMessage::MicromouseEvent(micromouse_event))
            .await;
        todo!("Send the event to the frontend and if it modifies the strategy_tree, also do that and send the necessary events for that (to the frontend and to the micromouse)")
    }

    #[instrument(
        skip(self),
        name = "handle_frontend_command",
        fields(description = "Handle a new command / strategy change etc. sent from the frontend")
    )]
    pub async fn handle_frontend_command(&mut self, frontend_command: FrontendResponse<N>) {
        todo!("Modify the strategy_tree or send the appropriate commands to the Micromouse")
    }
}
