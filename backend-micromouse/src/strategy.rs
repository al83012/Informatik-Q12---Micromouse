use crate::{ comm::micromouse::Command, map::PartialMap, position::Position, world_data::WorldData};


pub trait FromConfig {
    type Config;
    fn from_config(config: Self::Config) -> Self;
}

pub struct GoalPosition(pub Position);

pub struct ProblemState<const N: usize> {
    pub world_data: WorldData<N>,
    pub goal: GoalPosition
}

pub enum StrategyStepResult<const N: usize> {
    // There is no next step which leads closer to the problem resolution
    Impossible,
    Known(StrategyStep<N>),
    Unknown(Vec<StrategyStep<N>>),
}

pub struct StrategyStep<const N: usize> {
    cmds: Vec<Command>,
    end_state: PartialMap<N>,
}


pub trait Strategy : FromConfig {
    fn next_step<const N: usize>(&self, problem: ProblemState<N>) -> StrategyStepResult<N>;
}
