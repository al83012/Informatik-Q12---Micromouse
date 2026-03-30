use crate::{ comm::micromouse_message::Command, map::PartialMap, nonempty::NonEmptyVec, position::Position, world_data::WorldData};


pub trait FromConfig {
    type Config;
    fn from_config(config: Self::Config) -> Self;
}

pub struct GoalPosition(pub Position);

/// Same as WorldData, but signifies, that it is not the problem state at the end of a step, but
/// an incomplete look into the future (The contained map does not include all the information that
/// should be available at the position and rotation of the mouse, as this rotation and position is
/// yet to be reached)
pub struct PartialWorldData<const N: usize>(pub WorldData<N>);





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
    results: NonEmptyVec<CommandEndState<S, N>>
}

pub struct CommandEndState<S: Strategy, const N: usize> {
    internal_state: S,
    partial_world: PartialWorldData<N>
}


pub enum StrategyError {
    ImpossibleWithStrategy,
    Canceled,
    Other(String)
}

// While we normally just return 1 command (and its end-state), we can also clump them together (in
// order to streamline some strategies, such as wall-following)
pub type CommandSteps<S: Strategy, const N: usize> = NonEmptyVec<CommandStep<S, N>>;

pub type StrategyStepResult<S: Strategy, const N: usize> = Result<CommandSteps<S, N>, StrategyError>;


// Strategies are derived from configs (even if that config turns out to be ()), furthermore, they
// need to be sized as the CommandEndState also needs to create new copies of the Strategy Internal
// State
pub trait Strategy : FromConfig + Sized {

    /// Takes in an internal state and partial (or complete, if the micromouse caught up to the
    /// command scheduling) world data, trying to create a new command, which brings it closer to
    /// the goal
    fn moves_from_state<const N: usize>(self, world_data: impl Into<WorldData<N>>, goal: GoalPosition) -> StrategyStepResult<Self, N>;
}
