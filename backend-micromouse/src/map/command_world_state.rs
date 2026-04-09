// World state, which also stores the currently active command, so that one can simply call
// "step_to(n)" to step to the state of the command (automatically doing all the sub-steps)
//
// One should also be able to use all the partial and map functions (for instance measuring, etc.)
//
//
// Usecase:
// - If there is a new command / a command was started --> New
// - Every time a measurement or a cmd-finished is returned --> Call step_to(n) to bring the
// internal transform up to that step, but first: apply measurements
// - if cmd-finished: call ".finish" --> Returns the WorldData again --> can be used for the next
// command
//
//
// WARN:
// See notes on ipad

use std::collections::{hash_map, HashMap, HashSet};

use tungstenite::http::header::MaxSizeReached;

use crate::{
    comm::micromouse_message::{
        Command, InterruptOccurence, InterruptType, MeasurementInterrupt, MeasurementOccurence,
        StepNum, TransformedMovement,
    },
    map::{
        map::{Map, PartialMap},
        measurement::Measurement,
        upgrade::IsUpgradeable,
        world_data::{PartialWorldData, WorldData},
    },
};

pub struct FilteredCommandApplication<const N: usize> {
    // containing steps 0..<n
    // Not always at that length though
    // --> One ExecutionSubStep per
    // Still at least one per step (as every step, even without interrupt is recorded (to be able
    // to call .at_step without having to interpolate))
    execution_steps: Vec<ExecutionStep<N>>,

    // Either the last interrupt which is executed as it **HAS** to be triggered (based on filter)
    execution_termination: CommandTerminationReason<N>,
    last_possible_state: PartialWorldData<N>,
    // filter/what is currently actually known about the map --> doesn't modify the execution
    // steps, but can be used to filter the given paths
    // Does NOT include MouseTransform, as it is representative of all different steps during
    // command execution
    filter: Map<N>,

    transformed_move: TransformedMovement,

    command: Command,
}

/// Every step has an ExecutionStep, if there is no way that a step can fail, the
/// potential_interrupts are simply empty
pub struct ExecutionStep<const N: usize> {
    interrupts: Vec<PotentialCommandInterruptTermination<N>>,
}

// For a step: Could continue, but could also stop due to the interrupt
// --> Represents an interrupt that may trigger
pub struct PotentialCommandInterruptTermination<const N: usize> {
    // Could also still be a "continue" interrupt
    potentially_terminating_interrupt: InterruptType,
    interrupt_index: usize,
    // What the world would have to (at least) look like in order for the interrupt to not stop
    // execution
    // This includes the mouse transform as it is representative of a specific step (and thus
    // position) of execution
    continuing_world: PartialWorldData<N>,
    terminating_world: Option<PartialWorldData<N>>,
}

// For the very end of a command
// --> Represents an interrupt that will trigger
pub struct CommandInterruptEnd<const N: usize> {
    terminating_interrupt: InterruptType,
    interrupt_index: usize,
    terminating_world: PartialWorldData<N>,
}

// End of command
// --> Represents, whether the command ends due to reaching the end of its associated
// move-directive or if it is forced to terminate prematurely due to an interrupt
pub enum CommandTerminationReason<const N: usize> {
    Interrupted(CommandInterruptEnd<N>),
    MaxStep(usize),
}

/**
CommandApplication represents the execution of a command within a given filter.

The given command will precompute all the possible steps and outcomes.

Most methods will take the internal filter into account, like the `at_step_filtered` method,
which will just return `Unreachable` if the step cannot be reached with a given filter, `End`,
if it has to be the last step (given the filter), `Continue` if this is not the last step and
`PotentialEnd{end, continue}`, if there could be another step (as the filter is not disclosing
enough information)

The general workflow with this type:
- Taking in the currently known map as a filter, apply command
- Whenever there is new feedback / whenever a measurement comes in: Apply it to the filter
  --> Receive all the information about which interrupts / paths were pruned

And with that:
- For use in a strategy tree (where we do not really care about all the individual steps, but
  just the results of the commands): call `.potential_outcomes_given_filter` to get all the
  possible outcomes whenever the filter changes
  --> This will allow us to also update all the children-nodes in the strategy-tree (Also
  allowing us to use the `WorldData.intersect` function to
  combine all the nodes:

  (/potential outcomes of a command based on the potential outcomes of
  previous commands) within a level of the tree (As long as the intersects cannot agree on
  where the mouse is for instance: cannot build intersect, Maps are fundamentally different,
  but otherwise: can combine maps as long as they are potentially_eq (in which case they
  downgrade to the lowest common denominator)

  By combining all those nodes, as soon as a consensus is reached, the strategy can call `try_next_move()`
  on this combined map (which it is allowed to fail (as long as the only command left is
  not yet finished (as that would mean, that there will not be any additional information with which
  to work (until the next command is sent))))
- For use in simulation-environments:
  One generally has:
  - A map, which will contain the internally known full data
  - A partial world, the simulation-space

  With that, you can use the `FilteredCommandApplication`'s `measurements_at_step`-method to find out
  which measurements would be performed at a certain step,

  Then, these measurement-tasks (/directions) can be taken to actually perform the measurements using the
  map, which then get fed back into the FilteredCommandApplication.

  After this, we can actually call `at_step` to determine, whether the given measurements triggered an interrupt

*/
impl<const N: usize> FilteredCommandApplication<N> {
    /// Creates a new CommandApplication, which precomputes the steps a command takes to reach its
    /// execution end.
    ///
    /// `with_filter` is the world in which this execution will take place; It limits, which
    /// command-steps will be computed, as it will stop execution, if it violates the filter
    ///
    /// To get a "full" evaluation, set `with_filter` to `None` to use the Default (or empty /
    /// Undiscovered) World. This means, that any step with an interrupt, which could stop
    /// execution will find a way to change the filter so that it could stop at this point or not
    /// (The only exception to this is the case, in which interrupts are contradictory (like
    /// opposing `0_L_STOP-IF-BLOCKED` and `0_l_STOP-IF-OPEN`))
    pub fn new(with_filter: Option<WorldData<N>>, command: Command) -> Self {
        let with_filter = with_filter.unwrap_or_default();
        let filter = with_filter.map;

        let start_transform = with_filter.mouse;
        let transformed_move = TransformedMovement::new(command.ty, start_transform);

        let max_step_count = transformed_move.max_step_count();

        let mut execution_steps = Vec::with_capacity(max_step_count + 1);

        // The condition for the next step to happen is that **NO** interrupt was triggered before
        // that point --> next_step_start_requirements is the running toll of how the map would
        // have to look like to make it past a certain point
        let mut next_step_start_requirements = PartialWorldData::from(with_filter);

        for i in 0..=max_step_count {
            let transform_at_step = transformed_move.at_step(i).expect("step in range");

            // Encoding the info from transformed move, where the mouse is
            // --> Finally combining the map with movement
            next_step_start_requirements.mouse = transform_at_step;

            let mut step_potential_ends = vec![];

            for (interrupt_index, interrupt) in command.interrupts.iter().enumerate() {
                if !interrupt.at_step.matches(i) {
                    continue;
                }
                let potential_end_reason: InterruptType = interrupt.into();

                // Create a world-filter, which ensures, that the interrupt, which would trigger
                // the command to finish is off
                let continuing_world = next_step_start_requirements
                    .clone()
                    .with_interrupt_termination_triggered(
                        false,
                        potential_end_reason.direction,
                        potential_end_reason.action,
                    );

                let terminating_world = next_step_start_requirements.clone()
                    .with_interrupt_termination_triggered(
                        true,
                        potential_end_reason.direction,
                        potential_end_reason.action,
                    );

                // INFO: Adding the current interrupt as the end of this command execution --> HAS
                // to interrupt
                if continuing_world.is_none() {
                    // There is no way of passing this step without triggering an interrupt which
                    // will stop
                    // INFO: WILL HAVE TO STOP

                    let end_of_command =
                        CommandTerminationReason::Interrupted(CommandInterruptEnd {
                            terminating_interrupt: potential_end_reason,
                            interrupt_index,
                            terminating_world: terminating_world.expect("By definition, if `world_if_stop_NOT_triggered` does not exist, `world_if_stop_triggered` has to exist (As it is also the termination, it would be nice if it existed)"),
                        });

                    // Still need to add the current step to the list (but without the final
                    // interrupt)
                    execution_steps.push(ExecutionStep {
                        interrupts: step_potential_ends,
                    });
                    return Self {
                        execution_steps,
                        execution_termination: end_of_command,
                        filter,
                        command,
                        transformed_move,
                        last_possible_state: next_step_start_requirements
                    };
                }


                // INFO: Adding the current interrupt as a normal interrupt, which could activate
                // or not
                let continuing_world =
                    continuing_world.expect("Already handled forced end; This path **has** to continue");
                next_step_start_requirements = continuing_world.clone();

                // Even registers `Continue`-Interrupts (Which is why terminating world is optional)
                //
                step_potential_ends.push(PotentialCommandInterruptTermination {
                    potentially_terminating_interrupt: potential_end_reason,
                    continuing_world,
                    interrupt_index,
                    terminating_world,
                });
            }
            execution_steps.push(ExecutionStep {
                interrupts: step_potential_ends,
            });
        }

        Self {
            execution_steps,
            // The MaxStep-Execution end is only reached, if no interrupt is forced to trigger
            // before that (even if that interrupt is in the last step)
            execution_termination: CommandTerminationReason::MaxStep(max_step_count),
            filter,
            command,
            transformed_move,
            last_possible_state: next_step_start_requirements
        }
    }

    pub fn max_step(&self) -> usize {
        self.execution_steps.len()
    }

    pub fn update_filter(measurement: Measurement)
    /* -> Result --> can upgrade current filter to this? + Return all the newly discarded stops  */
    {
        todo!()

        //TODO: *self = Self::new(new_filter, self.command);
    }

    pub fn upgrade_filter(
        &mut self,
        upgraded_filter: Map<N>,
    ) -> Result<RejectedOutcomes, FilterUpgradeError> {
        let is_upgrade_valid = upgraded_filter.could_be_upgrade_of(&self.filter);

        if !is_upgrade_valid {
            return Err(FilterUpgradeError);
        }

        // for

        todo!()

        //TODO: *self = Self::new(new_filter, self.command);
    }

    fn potential_outcome_ids(&self) -> HashSet<PathLocalOutcomeId> {
        let mut current_potential_outcome_ids = HashSet::new();
        for (step_num, step) in self.execution_steps.iter().enumerate() {
            for pot_outcome in step.interrupts.iter() {
                let InterruptType { direction, action } =
                    pot_outcome.potentially_terminating_interrupt;
                if !action.could_interrupt() {
                    // Continue-Action; Not a potential Command-Outcome: Command cannot stop here
                    continue;
                }

                if pot_outcome.terminating_world.is_none() {
                    // Though an interrupt does exist and that interrupt generally could stop
                    // execution (it is not of type continue/was caught before), With the currently
                    // applied filter, it is no longer possible to interrupt at this point
                    continue;
                }

                current_potential_outcome_ids.insert(PathLocalOutcomeId {
                    at_step: step_num,
                    from_interrupt: PathLocalInterruptId::InterruptAtIndex(
                        pot_outcome.interrupt_index,
                    ),
                });
            }
        }

        // The guaranteed termination of the command, no interrupt before that may be forced
        match &self.execution_termination {
            CommandTerminationReason::Interrupted(i) => {
                let interrupt_idx = i.interrupt_index;
                current_potential_outcome_ids.insert(PathLocalOutcomeId {
                    at_step: self.step_with_termination(),
                    from_interrupt: PathLocalInterruptId::InterruptAtIndex(interrupt_idx),
                });
            }
            CommandTerminationReason::MaxStep(max_step) => {
                current_potential_outcome_ids.insert(PathLocalOutcomeId {
                    at_step: *max_step,
                    from_interrupt: PathLocalInterruptId::MaxStep,
                });
            }
        };

        current_potential_outcome_ids
    }

    // The max-step / last step which will (at least partially) be executed
    // The step at which the command either reaches the max_step_count of the move or is forced to
    // terminate by the filter
    pub fn step_with_termination(&self) -> usize {
        // The execution_steps go from 0..=step_with_termination
        self.execution_steps.len() - 1
    }

    /// Returns all the different outcomes that the execution of this command in the context of its
    /// current filter --> Can be used to construct a StrategyTree
    pub fn potential_outcomes_given_filter<'a>(&'a self) -> CommandOutcomes<'a, N> {
        let mut potential_outcomes = HashMap::new();

        // steps 0..<n
        for (step_num, step) in self.execution_steps.iter().enumerate() {
            for end in step.interrupts.iter() {
                let InterruptType { direction, action } = end.potentially_terminating_interrupt;
                if !action.could_interrupt() {
                    // Continue-Action; Not a potential Command-Outcome: Command cannot stop here
                    continue;
                }

                let world_if_stop_triggered = end.terminating_world.as_ref();

                if world_if_stop_triggered.is_none() {
                    // Though an interrupt does exist and that interrupt generally could stop
                    // execution (it is not of type continue/was caught before), With the currently
                    // applied filter, it is no longer possible to interrupt at this point

                    continue;
                }

                let termination_outcome =
                    world_if_stop_triggered.expect("Nonexistence of Outcome already handled");

                let index_of_interrupt = end.interrupt_index;

                let outcome_id = PathLocalOutcomeId {
                    at_step: step_num,
                    from_interrupt: PathLocalInterruptId::InterruptAtIndex(index_of_interrupt),
                };

                potential_outcomes.insert(outcome_id, termination_outcome);
            }
        }

        let termination = &self.execution_termination;
        match termination {
            CommandTerminationReason::Interrupted(command_interrupt_end) => {
                let outcome_id = PathLocalOutcomeId {
                    at_step: self.step_with_termination(),
                    from_interrupt: PathLocalInterruptId::InterruptAtIndex(
                        command_interrupt_end.interrupt_index,
                    ),
                };
                potential_outcomes.insert(outcome_id, &command_interrupt_end.terminating_world);
            }
            CommandTerminationReason::MaxStep(x) => {
                let outcome_id = PathLocalOutcomeId {
                    at_step: *x,
                    from_interrupt: PathLocalInterruptId::MaxStep,
                };


                potential_outcomes.insert(outcome_id, &self.last_possible_state);
            }
        }

        CommandOutcomes { potential_outcomes  }
    }

    // pub fn at_step(&self, )
}

pub struct FilterUpgradeError;

pub struct CommandOutcomes<'a, const N: usize> {
    pub potential_outcomes: HashMap<PathLocalOutcomeId, &'a PartialWorldData<N>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PathLocalOutcomeId {
    pub at_step: usize,
    pub from_interrupt: PathLocalInterruptId,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum PathLocalInterruptId {
    MaxStep,
    InterruptAtIndex(usize),
}

pub struct CommandApplicationIterator<const N: usize> {
    next_step: usize,
    application: FilteredCommandApplication<N>,
}

pub struct RejectedOutcomes {
    pub rejected_outcome_ids: HashSet<PathLocalOutcomeId>,
}
