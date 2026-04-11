use std::{ops::DerefMut, process::Command, sync::atomic::AtomicUsize};

use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex, Notify,
};

use crate::{
    comm::micromouse_message::StepNum,
    map::{
        command_world_state::FilteredCommandApplication,
        map::{Map, PartialMap},
    },
    transform::position::MouseTransform,
};

pub struct DirectSimulationEnvironment<const N: usize> {
    fully_discovered_map: Map<N>,
    discovered_map: PartialMap<N>,
    current_transform: MouseTransform,
    current_command: Mutex<Option<(FilteredCommandApplication<N>, usize)>>,
    command_queue_sender: Sender<Command>,
    command_queue_receiver: Receiver<Command>,
    empty_queue_notification: Notify,
}

impl<const N: usize> DirectSimulationEnvironment<N> {
    pub fn new(fully_discovered_map: Map<N>) -> Option<Self> {
        if !fully_discovered_map.is_fully_discovered() {
            return None;
        }

        let discovered_map = PartialMap::from(Map::default());
        let current_transform = MouseTransform::default();
        let current_command = Mutex::new(None);

        let (command_queue_sender, command_queue_receiver) = mpsc::channel(10);

        let empty_queue_notification = Notify::new();

        Some(Self {
            fully_discovered_map,
            discovered_map,
            current_transform,
            current_command,
            command_queue_sender,
            command_queue_receiver,
            empty_queue_notification,
        })
    }

    // Function that stops blocking / returns once a command is neccessary
    pub async fn command_required_interrupt(&self) {
        self.empty_queue_notification.notified().await
    }

    pub async fn queue_command(&self, command: Command) {
        let _ = self.command_queue_sender.send(command).await;
    }

    pub async fn step_command(&self) {
        let mut current_command = self.current_command.lock().await;
        let Some((current_command, next_step)) = current_command.deref_mut() else {
            self.empty_queue_notification.notify_one();
            return;
        };
        let measurements_to_perform =
            current_command.ordered_measurement_directions_at_step(*next_step as StepNum).expect("The command should be reachable as any termination should have been detected before");

        // let measurement_directions_to_perform = current_command.measurement_directions_at_step(next_step);

        todo!()
    }
}
