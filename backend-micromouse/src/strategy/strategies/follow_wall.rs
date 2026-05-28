use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    comm::micromouse_message::{
        Command, InterruptAction, InterruptStep, MeasurementInterrupt, MovementType,
    },
    strategy::strategy::{ComputedAction, ComputedActions, FromConfig, Strategy, StrategyEndState},
    transform::{direction::RelativeDirection, position::MouseTransform},
    utils::nonempty::NonEmpty,
};

#[derive(Clone, Debug)]
pub struct FollowWall<const N: usize> {
    next_move_id: usize,
    follow: WallDirection,

    // The list of places it has been when next_move_id % 4 == 0
    // If an element would occur twice, we have gone in a loop
    visited: HashSet<MouseTransform>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Copy, Serialize)]
pub enum WallDirection {
    Left,
    Right,
}

impl From<WallDirection> for RelativeDirection {
    fn from(value: WallDirection) -> Self {
        match value {
            WallDirection::Left => RelativeDirection::Left,
            WallDirection::Right => RelativeDirection::Right,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FollowWallConfig {
    pub follow_wall: WallDirection,
}

impl<const N: usize> FromConfig<N> for FollowWall<N> {
    type Config = FollowWallConfig;

    #[instrument(name = "from_config FollowWall", fields(description = "Create new FollowWall-Strategy instance from config"))]
    fn from_config(
        config: &Self::Config,
        _starting_state: &crate::map::world_data::WorldData<N>,
    ) -> Self {
        Self {
            next_move_id: 0,
            follow: config.follow_wall,
            visited: HashSet::new(),
        }
    }
}

impl<const N: usize> Strategy<N> for FollowWall<N> {
    #[instrument(name = "next_cmd FollowWall", fields(description = "Try to find next action after state"))]
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        if self.next_move_id.is_multiple_of(4) {
            // At forward searching move
            let current_pos = world.mouse;

            if self.visited.contains(&current_pos) {
                return crate::strategy::strategy::StrategyComputationResult::Computed(Err(StrategyEndState::NoPossibleAction("Already encountered this exact program state --> LOOP cannot be handled by only following one wall".to_string())));
            }
        }

        let mut successor = self.clone();

        if self.next_move_id.is_multiple_of(4) {
            successor.visited.insert(world.mouse);
        }

        successor.next_move_id = (self.next_move_id + 1) % 4;

        let do_command = self.command(self.next_move_id);

        crate::strategy::strategy::StrategyComputationResult::Computed(Ok(ComputedActions(
            NonEmpty::one(ComputedAction {
                next_strategy_state: Some(successor),
                after_command: do_command,
            }),
        )))
    }
}

impl<const N: usize> FollowWall<N> {
    #[instrument(name = "command", fields(description = "Access the command at the given index"))]
    pub fn command(&self, index: usize) -> Command {
        let always_right_commands: [Command; 4] = [
            Command {
                // Go forwards until you encounter a blockade or an opening on the right
                ty: MovementType::Move(N as u8),
                interrupts: vec![
                    MeasurementInterrupt {
                        direction: RelativeDirection::Forward,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfBlocked,
                    },
                    MeasurementInterrupt {
                        direction: RelativeDirection::Right,
                        at_step: InterruptStep::Each,
                        action: InterruptAction::StopIfOpen,
                    },
                ],
            },
            Command {
                ty: MovementType::Turn(-1),
                interrupts: vec![MeasurementInterrupt {
                    direction: RelativeDirection::Right,
                    at_step: InterruptStep::At(0),
                    action: InterruptAction::StopIfBlocked,
                }],
            },
            Command {
                ty: MovementType::Turn(2),
                interrupts: vec![MeasurementInterrupt {
                    direction: RelativeDirection::Forward,
                    at_step: InterruptStep::Each,
                    action: InterruptAction::StopIfOpen,
                }],
            },
            // Escape from turning point
            Command {
                ty: MovementType::Move(1),
                interrupts: vec![],
            },
        ];

        let mut cmd = always_right_commands[index % 4].clone();

        if self.follow == WallDirection::Left {
            // Mirror
            cmd.interrupts.iter_mut().for_each(|i| {
                if i.direction == RelativeDirection::Right {
                    i.direction = RelativeDirection::Left
                }
            });
            if let MovementType::Turn(i) = &mut cmd.ty {
                *i = *i * -1;
            }
        }

        cmd
    }
}
