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
    }, map::{Map, PartialMap}, measurement::Measurement, world_data::{PartialWorldData, WorldData}
};

pub struct CommandApplication<const N: usize> {
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

impl<const N: usize> CommandApplication<N> {
    pub fn new(world_at_start: WorldData<N>, command: Command) -> Self {
        let filter = world_at_start.map;

        let start_transform = world_at_start.mouse;
        let transformed_move = TransformedMovement::new(command.ty, start_transform);

        let max_step_count = transformed_move.max_step_count();

        let mut execution_steps = Vec::with_capacity(max_step_count + 1);

        // The condition for the next step to happen is that **NO** interrupt was triggered before
        // that point --> next_step_start_requirements is the running toll of how the map would
        // have to look like to make it past a certain point
        let mut next_step_start_requirements = PartialWorldData::from(world_at_start);

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

    pub fn update_filter(measurement: Measurement) /* -> Result */ {

    }

    // pub fn at_step(&self, )
}
