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

use crate::{
    comm::micromouse_message::{
        Command, InterruptOccurence, InterruptType, MeasurementInterrupt, MeasurementOccurence,
        StepNum, TransformedMovement,
    },
    map::{Map, PartialMap},
    measurement::Measurement,
    world_data::{PartialWorldData, WorldData},
};

pub struct FilteredCommandApplication<const N: usize> {
    // containing steps 0..<n
    // Not always at that length though
    // --> One ExecutionSubStep per
    // Still at least one per step (as every step, even without interrupt is recorded (to be able
    // to call .at_step without having to interpolate))
    execution_steps: Vec<ExecutionStep<N>>,
    // step n
    // condition = same as the the execution_steps[n-1] condition
    execution_end: CommandTerminationReason,
    // filter/what is currently actually known about the map --> doesn't modify the execution
    // steps, but can be used to filter the given paths
    // Does NOT include MouseTransform, as it is representative of all different steps during
    // command execution
    filter: Map<N>,
}

/// Every step has an ExecutionStep, if there is no way that a step can fail, the
/// potential_interrupts are simply empty
pub struct ExecutionStep<const N: usize> {
    potential_ends: Vec<PotentialExecutionEnd<N>>,
}

pub struct PotentialExecutionEnd<const N: usize> {
    // Could also still be a "continue" interrupt
    potential_end_reason: InterruptType,
    // What the world would have to (at least) look like in order for the interrupt to not stop
    // execution
    // This includes the mouse transform as it is representative of a specific step (and thus
    // position) of execution
    world_if_not_triggered: PartialWorldData<N>,
}

pub enum CommandTerminationReason {
    Interrupted(InterruptOccurence),
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

            for interrupt in &command.interrupts {
                if !interrupt.at_step.matches(i) {
                    continue;
                }
                let potential_end_reason: InterruptType = interrupt.into();

                // Create a world-filter, which ensures, that the interrupt, which would trigger
                // the command to finish is off
                let world_if_not_triggered = next_step_start_requirements
                    .with_interrupt_stop_triggered(
                        false,
                        potential_end_reason.direction,
                        potential_end_reason.action,
                    );

                if world_if_not_triggered.is_none() {
                    // There is no way of passing this step without triggering an interrupt which
                    // will stop

                    let end_of_command =
                        CommandTerminationReason::Interrupted(InterruptOccurence {
                            occurence: MeasurementOccurence {
                                direction: interrupt.direction,
                                at_step: i as StepNum,
                            },
                            action: interrupt.action,
                        });

                    return Self {
                        execution_steps,
                        execution_end: end_of_command,
                        filter,
                    };
                }
                let world_if_not_triggered =
                    world_if_not_triggered.expect("Already handled forced end");

                next_step_start_requirements = world_if_not_triggered.clone();

                step_potential_ends.push(PotentialExecutionEnd {
                    potential_end_reason,
                    world_if_not_triggered,
                });
            }
            execution_steps.push(ExecutionStep {
                potential_ends: step_potential_ends,
            });
        }

        Self {
            execution_steps,
            // The MaxStep-Execution end is only reached, if no interrupt is forced to trigger
            // before that (even if that interrupt is in the last step)
            execution_end: CommandTerminationReason::MaxStep(max_step_count),
            filter,
        }
    }

    pub fn max_step(&self) -> usize {
        self.execution_steps.len()
    }

    pub fn update_filter(measurement: Measurement)
    /* -> Result --> can upgrade current filter to this? + Return all the newly discarded stops  */
    {
    }

    /// Returns all the different outcomes that the execution of this command in the context of its
    /// current filter --> Can be used to construct a StrategyTree
    pub fn potential_outcomes_given_filter(&self) /* --> _ */ {}

    // pub fn at_step(&self, )
}

pub struct CommandApplicationIterator<const N: usize> {
    next_step: usize,
    application: FilteredCommandApplication<N>,
}
