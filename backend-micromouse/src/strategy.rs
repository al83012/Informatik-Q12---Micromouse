use std::{fmt::Display, ops::Deref};

use crate::{
    comm::micromouse_message::{Command, InterruptAction},
    direction::RelativeDirection,
    map::{PartialMap, WallDiscoveryStatus},
    nonempty::NonEmptyVec,
    position::{MouseTransform, Position},
    world_data::WorldData,
};

pub trait FromConfig {
    type Config;
    fn from_config(config: Self::Config) -> Self;
}

pub struct GoalPosition(pub Position);

/// Same as WorldData, but signifies, that it is not the problem state at the end of a step, but
/// an incomplete look into the future (The contained map does not include all the information that
/// should be available at the position and rotation of the mouse, as this rotation and position is
/// yet to be reached)
#[derive(Clone)]
pub struct PartialWorldData<const N: usize>(WorldData<N>);

impl<const N: usize> Display for PartialWorldData<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PARTIAL(pos = {:?}, dir = {:?})\n{}", self.0.mouse.pos, self.0.mouse.dir, self.0.map)
    }
}

impl<const N: usize> Deref for PartialWorldData<N> {
    type Target = WorldData<N>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> From<WorldData<N>> for PartialWorldData<N> {
    fn from(value: WorldData<N>) -> Self {
        Self(value)
    }
}

impl<const N: usize> PartialWorldData<N> {
    // Creates an alternate version of the PartialWorldData, which will ensure that the given
    // interrupt will trigger or not (depending on `should_trigger`) at the current transform
    // Returns None if the wall can never be set to a state which would trigger the interrupt
    // TODO: confirm
    pub fn with_interrupt_triggered(
        mut self,
        should_trigger: bool,
        interrupt_dir: RelativeDirection,
        condition: InterruptAction,
    ) -> Option<Self> {
        let current_pos = self.0.mouse.pos;
        let checked_dir = &interrupt_dir.transform_by(&self.0.mouse.dir);

        let deciding_wall = (&mut self.0.map).wall_mut(&current_pos, checked_dir)?;
        *deciding_wall = match (*deciding_wall, condition) {
            // Interrupt will never be triggered
            (_, InterruptAction::Continue) => {
                return if should_trigger { None } else { Some(self) }
            }

            // Interrupt can be triggered
            (WallDiscoveryStatus::Undiscovered, InterruptAction::StopIfBlocked) => {
                WallDiscoveryStatus::Exists(should_trigger)
            }
            (WallDiscoveryStatus::Undiscovered, InterruptAction::StopIfOpen) => {
                WallDiscoveryStatus::Exists(!should_trigger)
            }

            // Interrupt will be triggered
            (WallDiscoveryStatus::Exists(true), InterruptAction::StopIfBlocked) => {
                if should_trigger {
                    *deciding_wall
                } else {
                    return None;
                }
            }
            (WallDiscoveryStatus::Exists(false), InterruptAction::StopIfOpen) => {
                if should_trigger {
                    *deciding_wall
                } else {
                    return None;
                }
            }

            // Interrupt will never be triggered
            (WallDiscoveryStatus::Exists(false), InterruptAction::StopIfBlocked) => {
                if !should_trigger {
                    *deciding_wall
                } else {
                    return None;
                }
            }
            (WallDiscoveryStatus::Exists(true), InterruptAction::StopIfOpen) => {
                if !should_trigger {
                    *deciding_wall
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        Some(self)
    }

    pub fn new(partial_map: PartialMap<N>, mouse_transform: MouseTransform) -> Self {
        Self(WorldData { map: partial_map.0, mouse: mouse_transform })
    }
}

// pub enum StrategyStepResult<const N: usize> {
//     // There is no next step which leads closer to the problem resolution
//     Impossible,
//     Known(StrategyStep<N>),
//     Unknown(Vec<StrategyStep<N>>),
// }
//
// pub struct StrategyStep<const N: usize> {
//     cmds: Vec<Command>,
//     end_state: PartialMap<N>,
// }

pub struct CommandStep<S: Strategy, const N: usize> {
    command: Command,
    /// One single command may have multiple end-states due to interrupts
    results: NonEmptyVec<CommandEndState<S, N>>,
}

pub struct CommandEndState<S: Strategy, const N: usize> {
    internal_state: S,
    partial_world: PartialWorldData<N>,
}

pub enum StrategyError {
    ImpossibleWithStrategy,
    Canceled,
    Other(String),
}

// While we normally just return 1 command (and its end-state), we can also clump them together (in
// order to streamline some strategies, such as wall-following)
pub type CommandSteps<S: Strategy, const N: usize> = NonEmptyVec<CommandStep<S, N>>;

pub type StrategyStepResult<S: Strategy, const N: usize> =
    Result<CommandSteps<S, N>, StrategyError>;

// Strategies are derived from configs (even if that config turns out to be ()), furthermore, they
// need to be sized as the CommandEndState also needs to create new copies of the Strategy Internal
// State
pub trait Strategy: FromConfig + Sized {
    /// Takes in an internal state and partial (or complete, if the micromouse caught up to the
    /// command scheduling) world data, trying to create a new command, which brings it closer to
    /// the goal
    fn moves_from_state<const N: usize>(
        self,
        world_data: impl Into<WorldData<N>>,
        goal: GoalPosition,
    ) -> StrategyStepResult<Self, N>;
}
