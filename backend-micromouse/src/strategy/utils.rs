use crate::{
    comm::micromouse_message::Command,
    strategy::strategy::{ComputedAction, Strategy},
};

impl<const N: usize, S: Strategy<N>> ComputedAction<N, S> {
    pub fn new_compound_substep(do_command: Command) -> Self {
        Self {
            next_strategy_state: None,
            after_command: do_command,
        }
    }
    pub fn new_step(do_command: Command, state_after: S) -> Self {
        Self {
            next_strategy_state: Some(state_after),
            after_command: do_command,
        }
    }
}
