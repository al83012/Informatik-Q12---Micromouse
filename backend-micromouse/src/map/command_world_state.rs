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

use std::{
    collections::{HashMap, HashSet},
    ops::Sub,
};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{
    comm::{
        micromouse_message::{Command, InterruptType, MovementType, StepNum, TransformedMovement},
        website::DiscoveryMessage,
    },
    map::{
        map::{CellDiscoveryStatus, Map, MapInconsistencyError, WallDiscoveryStatus},
        measurement::Measurement,
        upgrade::IsUpgradeable,
        world_data::{PartialWorldData, WorldData},
    },
    transform::direction::RelativeDirection,
    utils::nonempty::{NonEmpty, PotentiallyNonEmpty},
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

    potential_outcome_ids: CommandOutcomeIds,

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
        info!(target: "map/cmd/apl", "NEW Command Application for {command:?}");
        if let Some(f) = &with_filter {
            info!(target: "map/cmd/apl", "WITH FILTER\n{}", f);
        }
        let with_filter = with_filter.unwrap_or_default();
        let filter = with_filter.map;

        let start_transform = with_filter.mouse;
        let transformed_move = TransformedMovement::new(command.ty, start_transform);

        let max_step_count = transformed_move.max_step_count();

        let mut execution_steps = Vec::with_capacity(max_step_count + 1);

        let mut potential_outcome_ids = HashSet::new();

        // The condition for the next step to happen is that **NO** interrupt was triggered before
        // that point --> next_step_start_requirements is the running toll of how the map would
        // have to look like to make it past a certain point
        let mut next_step_start_requirements = PartialWorldData::from(with_filter);

        debug!(target: "map/cmd/apl", "  >> Could reach {max_step_count}?");
        for i in 0..=max_step_count {
            debug!(target: "map/cmd/apl", "     >> Processing step {i}");
            let transform_at_step = transformed_move.at_step(i).expect("step in range");
            debug!(target: "map/cmd/apl", "     >> Transform at step = {transform_at_step:?}");

            // Mark current cell as visited
            if let Some(cell) = next_step_start_requirements
                .map
                .cell_mut(&transform_at_step.pos)
            {
                *cell = CellDiscoveryStatus::Visited;
            }
            if i >= 1 {
                if let MovementType::Move(_) = command.ty {
                    let current = transform_at_step.pos;
                    let move_dir = transform_at_step.dir;
                    let mark_dir = move_dir.rotated(2);
                    if let Some(wall) = next_step_start_requirements
                        .map
                        .wall_mut(&current, &mark_dir)
                    {
                        debug!(target: "map/cmd/apl", "Marking {current} & {mark_dir} as visited");
                        *wall = WallDiscoveryStatus::Visited;
                    }
                }
            }
            //
            // {
            // if i > 0 {
            //     if let MovementType::Move(_) = command.ty {
            //         let current = next_step_start_requirements.mouse.pos;
            //         let move_dir = next_step_start_requirements.mouse.dir;
            //         let mark_dir = move_dir.rotated(2);
            //         if let Some(wall) = next_step_start_requirements
            //             .map
            //             .wall_mut(&current, &mark_dir)
            //         {
            //             *wall = WallDiscoveryStatus::Visited;
            //         }
            //     }
            // }

            // Encoding the info from transformed move, where the mouse is
            // --> Finally combining the map with movement
            next_step_start_requirements.mouse = transform_at_step;

            let mut step_potential_ends = vec![];

            for (interrupt_index, interrupt) in command.interrupts.iter().enumerate() {
                if !interrupt.at_step.matches(i) {
                    continue;
                }
                debug!(target: "map/cmd/apl", "         >> Interrupt [{interrupt_index}] = {interrupt:?}");
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

                let terminating_world = next_step_start_requirements
                    .clone()
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

                    debug!(target: "map/cmd/apl", "         >> NO CONTINUING WORLD");

                    if let Some(terminating_world) = &terminating_world {
                        //debug!(target: "map/cmd/apl", "             >> TERMINATING_WORLD = \n{terminating_world}");
                        debug!(target: "map/cmd/apl", "        >> BUT FOUND TERMINATING WORLD");
                    } else {
                        error!(target: "map/cmd/apl", "             >> NO TERMINATING_WORLD EITHER");
                    }

                    let end_of_command =
                        CommandTerminationReason::Interrupted(CommandInterruptEnd {
                            terminating_interrupt: potential_end_reason,
                            interrupt_index,
                            terminating_world: terminating_world.expect("By definition, if `world_if_stop_NOT_triggered` does not exist, `world_if_stop_triggered` has to exist (As it is also the termination, it would be nice if it existed)"),
                        });

                    debug!(target: "map/cmd/apl", "         >> Command finishes through interrupt --> FORCED");
                    // Still need to add the current step to the list (but without the final
                    // interrupt)
                    execution_steps.push(ExecutionStep {
                        interrupts: step_potential_ends,
                    });
                    potential_outcome_ids.insert(PathLocalOutcomeId {
                        at_step: i,
                        from_interrupt: PathLocalInterruptId::InterruptAtIndex(interrupt_index),
                    });
                    return Self {
                        execution_steps,
                        execution_termination: end_of_command,
                        filter,
                        command,
                        transformed_move,
                        last_possible_state: next_step_start_requirements,
                        potential_outcome_ids: CommandOutcomeIds {
                            potential_outcome_ids,
                        },
                    };
                }

                // INFO: Adding the current interrupt as a normal interrupt, which could activate
                // or not
                let continuing_world = continuing_world
                    .expect("Already handled forced end; This path **has** to continue");
                // debug!(target: "map/cmd/apl", "CONTINUING WORLD {interrupt} at step {i}\n{continuing_world}");
                debug!(target: "map/cmd/apl", "CONTINUING WORLD at step {i}\n{continuing_world}");
                next_step_start_requirements = continuing_world.clone();

                // Even registers `Continue`-Interrupts (Which is why terminating world is optional)
                //
                debug!(target: "map/cmd/apl", "        >> WITH CONTINUING WORLD");
                if terminating_world.is_some() {
                    let interrupt_id = PathLocalInterruptId::InterruptAtIndex(interrupt_index);
                    potential_outcome_ids.insert(PathLocalOutcomeId {
                        at_step: i,
                        from_interrupt: interrupt_id,
                    });
                    debug!(target: "map/cmd/apl", "        >> INTERRUPT_INDEX = {interrupt_id:?}");
                }
                step_potential_ends.push(PotentialCommandInterruptTermination {
                    potentially_terminating_interrupt: potential_end_reason,
                    continuing_world,
                    interrupt_index,
                    terminating_world,
                });
            }
            debug!(target: "map/cmd/apl", "    >> Finished Step");
            execution_steps.push(ExecutionStep {
                interrupts: step_potential_ends,
            });
        }

        debug!(target: "map/cmd/apl", "    >> MAX STEP at end of {max_step_count}");
        potential_outcome_ids.insert(PathLocalOutcomeId {
            at_step: max_step_count,
            from_interrupt: PathLocalInterruptId::MaxStep,
        });

        Self {
            execution_steps,
            // The MaxStep-Execution end is only reached, if no interrupt is forced to trigger
            // before that (even if that interrupt is in the last step)
            execution_termination: CommandTerminationReason::MaxStep(max_step_count),
            filter,
            command,
            transformed_move,
            last_possible_state: next_step_start_requirements,
            potential_outcome_ids: CommandOutcomeIds {
                potential_outcome_ids,
            },
        }
    }

    pub fn command(&self) -> &Command {
        &self.command
    }

    // pub fn max_step(&self) -> usize {
    //     self.execution_steps.len()
    // }

    pub fn command_unfiltered_max_step(&self) -> usize {
        self.command.max_step_count()
    }

    pub fn apply_measurement_to_filter(
        &mut self,
        measurement: Measurement,
    ) -> Result<FilterUpdate, FilterMeasurementUpgradeError> {
        debug!(target: "map/cmd/apl", "APPLYING MEASUREMENT TO FILTER");
        let discoveries = self.filter.apply_measurement(&measurement)?.non_empty();
        let rejections = self.upgrade_filter(self.filter)?.non_empty();

        Ok(FilterUpdate {
            discoveries,
            rejections,
        })
    }

    pub fn upgrade_filter(
        &mut self,
        upgraded_filter: Map<N>,
    ) -> Result<RejectedOutcomes, FilterUpgradeError> {
        debug!(target: "map/cmd/apl", "UPGRADING FILTER \n{} \nwith \n{}", self.filter, upgraded_filter);
        let is_upgrade_valid = upgraded_filter.could_be_upgrade_of(&self.filter);

        if !is_upgrade_valid {
            error!(target: "map/cmd/apl", "UPGRADE INVALID");
            return Err(FilterUpgradeError);
        }

        let new_filter = WorldData {
            map: upgraded_filter,
            mouse: self
                .transformed_move
                .at_step(0)
                .expect("Step 0 is always valid"),
        };
        let old_outcomes = self.potential_outcome_ids();

        // for

        // todo!()

        *self = Self::new(Some(new_filter), self.command.clone());
        let new_outcomes = self.potential_outcome_ids();
        let rejected = old_outcomes - new_outcomes;
        Ok(rejected)
    }

    pub fn potential_outcome_ids(&self) -> CommandOutcomeIds {
        let mut current_potential_outcome_ids = HashSet::new();
        for (step_num, step) in self.execution_steps.iter().enumerate() {
            for pot_outcome in step.interrupts.iter() {
                let InterruptType {
                    direction: _,
                    action,
                } = pot_outcome.potentially_terminating_interrupt;
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

        CommandOutcomeIds {
            potential_outcome_ids: current_potential_outcome_ids,
        }
    }

    // The max-step / last step which will (at least partially) be executed
    // The step at which the command either reaches the max_step_count of the move or is forced to
    // terminate by the filter
    pub fn step_with_termination(&self) -> usize {
        // The execution_steps go from 0..=step_with_termination
        // Can do this as even steps without interrupts are noted
        self.execution_steps.len() - 1
    }

    /// Returns all the different outcomes that the execution of this command in the context of its
    /// current filter --> Can be used to construct a StrategyTree
    pub fn potential_outcomes_given_filter<'a>(&'a self) -> CommandOutcomes<'a, N> {
        let mut potential_outcomes = HashMap::new();

        // steps 0..<n
        for (step_num, step) in self.execution_steps.iter().enumerate() {
            for end in step.interrupts.iter() {
                let InterruptType {
                    direction: _,
                    action,
                } = end.potentially_terminating_interrupt;
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

        CommandOutcomes { potential_outcomes }
    }

    // Will only return Ok, if the path to the given step is certain
    //
    // Will return the starting condition of that step (without any measurements applied)
    pub fn at_start_certain_step(
        &self,
        step_number: StepNum,
    ) -> Result<PartialWorldData<N>, CertainStepError> {
        debug!(target: "map/cmd/apl", "DETERMINING START OF STEP {step_number}");
        self.reach_step(step_number)?;

        // All steps before the one we are targetting have to be non-interrupted (at this point)
        // -->
        for i in 0..step_number {
            let step = self
                .execution_steps
                .get(i as usize)
                .expect("Step has to exist");
            for interrupt in step.interrupts.iter() {
                // WARN: Found a branching step in the past
                if interrupt.terminating_world.is_some() {
                    error!(target: "map/cmd/apl", "    FOUND BRANCH AT STEP {i} --> Not proven");
                    return Err(CertainStepError::Uncertainty(UncertainStepError {
                        tried_to_reach_step: step_number,
                        but_could_terminate_at: i,
                    }));
                }
            }
        }

        Ok(if step_number == 0 {
            debug!(target: "map/cmd/apl", "    STEP = 0 --> start = filter");
            PartialWorldData::new(
                self.filter.into(),
                self.transformed_move
                    .at_step(0)
                    .expect("Step 0 always valid"),
            )
        } else {
            debug!(target: "map/cmd/apl", "    STEP != 0 --> start = end_{{step-1}}");
            let max_step_before = self.max_substep_in_step(step_number - 1)?;
            // The previous step has to be continuing
            assert!(max_step_before.potential_termination == MaxSubstepTermination::Continuing);
            let mut map_continued = max_step_before.world_at_substep.map;

            let mouse_transf_at_step = self
                .transformed_move
                .at_step(step_number as usize)
                .expect("Already checked");

            if let Some(cell) = map_continued.cell_mut(&mouse_transf_at_step.pos) {
                *cell = CellDiscoveryStatus::Visited;
            }
            if let MovementType::Move(_) = self.transformed_move.movement {
                let backward = mouse_transf_at_step.dir.rotated(2);
                if let Some(wall) = map_continued.wall_mut(&mouse_transf_at_step.pos, &backward) {
                    // Is ok, as step > 0
                    *wall = WallDiscoveryStatus::Visited;
                }
            }
            PartialWorldData::new(map_continued.into(), mouse_transf_at_step)
        })

        // todo!()
    }

    pub fn max_substep_in_step(
        &self,
        step_number: StepNum,
    ) -> Result<MaxSubstep<N>, CannotReachStep> {
        debug!(target: "map/cmd/apl", "DETERMINING MAX SUBSTEP IN STEP {step_number}");
        self.reach_step(step_number)?;
        let step = self
            .execution_steps
            .get(step_number as usize)
            .expect("Already checked");
        let potential_terminations = &step.interrupts;
        if potential_terminations.is_empty() {
            debug!(target: "map/cmd/apl", "    NO BRANCHES IN CURRENT STEP --> Need to find some form of continuing-world in previous steps");
            // Have to look at previous steps
            // This step did not contain any map-updates or measurements
            if step_number == 0 {
                debug!(target: "map/cmd/apl", "        STEP = 0; World = filter");
                // There is no previous step
                return Ok(MaxSubstep {
                    potential_termination: if step_number as usize == self.step_with_termination() {
                        //There is only step 0
                        MaxSubstepTermination::Terminated(PathLocalInterruptId::MaxStep)
                    } else {
                        MaxSubstepTermination::Continuing
                    },
                    world_at_substep: WorldData {
                        map: self.filter,
                        mouse: self
                            .transformed_move
                            .at_step(0)
                            .expect("Step 0 is always valid"),
                    }
                    .into(),
                });
            }
            let mut last_continuing = None;
            for step_before_idx in step_number as usize - 1..=0 {
                let step_before = self
                    .execution_steps
                    .get(step_before_idx)
                    .expect("Smaller than checked step_num");
                if step_before.interrupts.is_empty() {
                    // This interrupt has not had any measurements either
                    continue;
                }
                debug!(target: "map/cmd/apl", "        Last continuing world at step {step_before_idx}");
                last_continuing = Some(
                    step_before
                        .interrupts
                        .last()
                        .expect("Already checked")
                        .continuing_world
                        .map(),
                );
                break;
            }
            // The last known map during command execution --> Is all the information we have about
            // this step's map
            let last_continuing = last_continuing.unwrap_or(self.filter.into());
            Ok(MaxSubstep {
                potential_termination: if step_number as usize == self.step_with_termination() {
                    MaxSubstepTermination::Terminated(PathLocalInterruptId::MaxStep)
                } else {
                    MaxSubstepTermination::Continuing
                },
                world_at_substep: WorldData {
                    map: last_continuing.into(),
                    mouse: self
                        .transformed_move
                        .at_step(step_number as usize)
                        .expect("Already checked"),
                }
                .into(),
            })
        } else {
            // There are worlds in this step

            debug!(target: "map/cmd/apl", "    Contains substeps");
            if step_number as usize == self.step_with_termination() {
                debug!(target: "map/cmd/apl", "        Is the step with termination --> last_possible_state = last_possible_state");
                // This step is the last step --> The last substep is some form of interruption
                let last_state = self.last_possible_state.clone();
                Ok(MaxSubstep {
                    potential_termination: MaxSubstepTermination::Terminated(
                        match &self.execution_termination {
                            CommandTerminationReason::Interrupted(i) => {
                                PathLocalInterruptId::InterruptAtIndex(i.interrupt_index)
                            }
                            CommandTerminationReason::MaxStep(_i) => PathLocalInterruptId::MaxStep,
                        },
                    ),
                    world_at_substep: last_state,
                })
            } else {
                debug!(target: "map/cmd/apl", "        Get last substep's continuing world");
                let last_world = potential_terminations
                    .last()
                    .expect("vec nonempty")
                    .continuing_world
                    .clone();

                Ok(MaxSubstep {
                    potential_termination: MaxSubstepTermination::Continuing,
                    world_at_substep: last_world,
                })
            }
        }
    }

    pub fn reach_step(&self, step_number: StepNum) -> Result<(), CannotReachStep> {
        if step_number > self.step_with_termination() as u32 {
            error!(target: "map/cmd/apl", "{step_number} > TERMINATION_STEP = {}", self.step_with_termination());
            Err(CannotReachStep(step_number))
        } else {
            Ok(())
        }
    }

    pub fn measurement_directions_at_step(
        &self,
        step_number: StepNum,
    ) -> Result<HashSet<RelativeDirection>, CannotReachStep> {
        self.reach_step(step_number)?;
        let mut directions = HashSet::new();
        for i in self.command.interrupts.iter() {
            if i.at_step.matches(step_number as usize) {
                directions.insert(i.direction);
            }
        }
        Ok(directions)
    }

    /// For use in simulation-environments --> Will give the measurement-directives in an ordered
    /// form, so that we can properly terminate early if an interrupt is triggered (that behaviour
    /// isn't useful, but it is closest to what we have to do)
    /// Returns every relative direction with their associated interrupt_index in ascending order
    pub fn ordered_measurement_directions_at_step(
        &self,
        step_number: StepNum,
    ) -> Result<Vec<(usize, RelativeDirection)>, CannotReachStep> {
        self.reach_step(step_number)?;
        let mut directions = Vec::new();
        for (i_num, i) in self.command.interrupts.iter().enumerate() {
            if i.at_step.matches(step_number as usize) {
                directions.push((i_num, i.direction));
            }
        }
        Ok(directions)
    }
}

pub struct MaxSubstep<const N: usize> {
    potential_termination: MaxSubstepTermination,
    // transf. is the same as the entire step
    // map matches that of the last possible continuing_world
    world_at_substep: PartialWorldData<N>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaxSubstepTermination {
    // The substep is the very last possible continuing_world of the step since it never stopped
    Continuing,
    Terminated(PathLocalInterruptId),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CertainStepError {
    CannotReachStep(CannotReachStep),
    Uncertainty(UncertainStepError),
}

impl From<CannotReachStep> for CertainStepError {
    fn from(value: CannotReachStep) -> Self {
        Self::CannotReachStep(value)
    }
}
impl From<UncertainStepError> for CertainStepError {
    fn from(value: UncertainStepError) -> Self {
        Self::Uncertainty(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UncertainStepError {
    pub tried_to_reach_step: StepNum,
    pub but_could_terminate_at: StepNum,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterUpgradeError;

#[derive(Debug, Serialize, Deserialize)]
pub enum FilterMeasurementUpgradeError {
    NotValidUpgrade(FilterUpgradeError),
    NotValidMeasurement(MapInconsistencyError),
}

impl From<MapInconsistencyError> for FilterMeasurementUpgradeError {
    fn from(value: MapInconsistencyError) -> Self {
        Self::NotValidMeasurement(value)
    }
}
impl From<FilterUpgradeError> for FilterMeasurementUpgradeError {
    fn from(value: FilterUpgradeError) -> Self {
        Self::NotValidUpgrade(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CannotReachStep(pub StepNum);

pub struct CommandOutcomes<'a, const N: usize> {
    pub potential_outcomes: HashMap<PathLocalOutcomeId, &'a PartialWorldData<N>>,
}

#[derive(Debug)]
pub struct CommandOutcomeIds {
    pub potential_outcome_ids: HashSet<PathLocalOutcomeId>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct PathLocalOutcomeId {
    pub at_step: usize,
    pub from_interrupt: PathLocalInterruptId,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum PathLocalInterruptId {
    MaxStep,
    InterruptAtIndex(usize),
}

// TODO:
pub struct CommandApplicationIterator<const N: usize> {
    next_step: usize,
    application: FilteredCommandApplication<N>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RejectedOutcomes {
    pub rejected_outcome_ids: HashSet<PathLocalOutcomeId>,
}

#[derive(Debug)]
pub struct FilterUpdate {
    pub discoveries: Option<NonEmpty<DiscoveryMessage>>,
    pub rejections: Option<NonEmpty<RejectedOutcomes>>,
}

impl Sub for CommandOutcomeIds {
    type Output = RejectedOutcomes;

    fn sub(self, rhs: Self) -> Self::Output {
        RejectedOutcomes {
            rejected_outcome_ids: self
                .potential_outcome_ids
                .difference(&rhs.potential_outcome_ids)
                .cloned()
                .collect(),
        }
    }
}

impl PotentiallyNonEmpty for RejectedOutcomes {
    fn is_empty(&self) -> bool {
        self.rejected_outcome_ids.is_empty()
    }
}

pub struct LazyFilteredCommandApplication<const N: usize> {
    pub command: Command,
    pub in_world: WorldData<N>,
}

impl<const N: usize> From<LazyFilteredCommandApplication<N>> for FilteredCommandApplication<N> {
    fn from(value: LazyFilteredCommandApplication<N>) -> Self {
        Self::new(Some(value.in_world), value.command)
    }
}
