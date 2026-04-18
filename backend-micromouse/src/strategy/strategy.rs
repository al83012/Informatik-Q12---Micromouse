use crate::{
    comm::micromouse_message::Command,
    map::world_data::{PartialWorldData, WorldData},
    transform::position::Position,
    utils::nonempty::NonEmpty,
};

#[derive(Clone, Debug)]
pub enum StrategyEndState {
    NoPossibleAction(String),
    ReachedGoal
}

pub enum StrategyComputationResult<const N: usize, S: Strategy<N>> {
    NotEnoughInformation,
    Computed(Result<ComputedActions<N, S>, StrategyEndState>),
}

// pub type ComputedActions<const N: usize, S: Strategy<N>> = NonEmpty<Vec<ComputedAction<N, S>>>;
pub struct ComputedActions<const N: usize, S: Strategy<N>>(pub NonEmpty<Vec<ComputedAction<N, S>>>);

// The action that was computed from certain starting world state; Contains the next_strategy_state
// after completing that action
pub struct ComputedAction<const N: usize, S: Strategy<N>> {
    // If the next_strategy_state is None, it means that this state is not meant to compute a next
    // step (It was likely created as the result of a batched action)
    pub next_strategy_state: Option<S>,
    pub after_command: Command,
}

#[derive(Clone, Copy, Debug)]
pub struct GoalPosition(pub Position);
pub trait FromConfig {
    type Config;
    fn from_config(config: Self::Config) -> Self;
}

pub trait Strategy<const N: usize>: FromConfig + Sized {
    fn next_cmd(
        &self,
        world: &PartialWorldData<N>,
        goal: &GoalPosition,
    ) -> StrategyComputationResult<N, Self>;
}

pub trait SerializeInformationView<const N: usize>: Strategy<N> {
    fn format_information_msg(&self) -> String;
}
