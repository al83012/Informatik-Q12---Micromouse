use std::{fmt::Display, ops::Deref};

use crate::{
    comm::micromouse_message::{
        Command, InterruptAction, MeasurementInterrupt, StepNum, TransformedCommand,
    },
    direction::{Direction, RelativeDirection},
    map::{self, Map, PartialMap, WallDiscoveryStatus},
    measurement::{self, Measurement},
    position::MouseTransform,
};

#[derive(Clone)]
pub struct WorldData<const N: usize> {
    pub map: Map<N>,
    pub mouse: MouseTransform,
}

impl<const N: usize> Default for WorldData<N> {
    fn default() -> Self {
        Self {
            map: Map::<N>::new(),
            mouse: MouseTransform::default(),
        }
    }
}

impl<const N: usize> WorldData<N> {
    pub fn apply_measurement(
        &mut self,
        measurement: &Measurement,
    ) -> Result<crate::comm::website::DiscoveryMessage, map::MapInconsistencyError> {
        self.map.apply_measurement(measurement)
    }
    pub fn measure_one(&self, relative_direction: RelativeDirection) -> &WallDiscoveryStatus {
        let measure_dir = relative_direction.transform_by(&self.mouse.dir);
        if (self.mouse.pos.x == 0 && measure_dir == Direction::NegX)
            || (self.mouse.pos.y == 0 && measure_dir == Direction::NegY)
        {
            return &WallDiscoveryStatus::Exists(true);
        }
        self.map
            .wall(
                &self.mouse.pos,
                &relative_direction.transform_by(&self.mouse.dir),
            )
            .expect("")
    }
    pub fn measure(&self, relative_direction: RelativeDirection, max_depth: u8) -> Measurement {
        let start_pos = self.mouse;
        for i in 0..=max_depth {
            let current_pos = start_pos.moved(i);
            if current_pos.is_none() {
                return Measurement {
                    value: measurement::MeasurementValue::Value { cells: i as u32 },
                    direction: relative_direction.transform_by(&start_pos.dir),
                    position: start_pos.pos,
                };
            }
            let current_pos = current_pos.unwrap();
            let next_wall = self
                .map
                .wall(&current_pos.pos, &start_pos.dir)
                .expect("Already checked");
            if i != max_depth {
                // Not yet the end --> could continue
                match next_wall {
                    //INFO: The ray only doesn't hit a wall if it is explicitly not there
                    //Does not work, if it is the max-depth: int that case HAS to create a measurement
                    WallDiscoveryStatus::Exists(false) => continue,
                    WallDiscoveryStatus::Exists(true) => {
                        return Measurement {
                            value: measurement::MeasurementValue::Value { cells: i as u32 },
                            direction: start_pos.dir,
                            position: start_pos.pos,
                        };
                    }
                    WallDiscoveryStatus::Undiscovered => {
                        return Measurement {
                            value: measurement::MeasurementValue::OutsideRange {
                                at_least_cells: i as u32,
                            },
                            direction: relative_direction.transform_by(&start_pos.dir),
                            position: start_pos.pos,
                        };
                    }
                }
            } else {
                match next_wall {
                    //
                    WallDiscoveryStatus::Exists(false) => {
                        return Measurement {
                            value: measurement::MeasurementValue::OutsideRange {
                                at_least_cells: i as u32,
                            },
                            direction: relative_direction.transform_by(&start_pos.dir),
                            position: start_pos.pos,
                        };
                    }
                    WallDiscoveryStatus::Exists(true) => {
                        return Measurement {
                            value: measurement::MeasurementValue::Value { cells: i as u32 },
                            direction: start_pos.dir,
                            position: start_pos.pos,
                        };
                    }
                    WallDiscoveryStatus::Undiscovered => {
                        return Measurement {
                            value: measurement::MeasurementValue::OutsideRange {
                                at_least_cells: i as u32,
                            },
                            direction: relative_direction.transform_by(&start_pos.dir),
                            position: start_pos.pos,
                        };
                    }
                }
            }
        }
        Measurement {
            value: measurement::MeasurementValue::OutsideRange {
                at_least_cells: max_depth as u32,
            },
            direction: start_pos.dir,
            position: start_pos.pos,
        }
    }

    pub fn is_interrupt_triggered(
        &self,
        interrupt: MeasurementInterrupt,
        at_step: StepNum,
    ) -> bool {
        if !interrupt.at_step.matches(at_step as usize) {
            return false;
        }
        let wall = self.measure_one(interrupt.direction);

        matches!(
            (wall, interrupt.action),
            (
                WallDiscoveryStatus::Exists(true),
                InterruptAction::StopIfBlocked
            ) | (
                WallDiscoveryStatus::Exists(false),
                InterruptAction::StopIfOpen
            )
        )
    }
}

/// Same as WorldData, but signifies, that it is not the problem state at the end of a step, but
/// an incomplete look into the future (The contained map does not include all the information that
/// should be available at the position and rotation of the mouse, as this rotation and position is
/// yet to be reached)
#[derive(Clone)]
pub struct PartialWorldData<const N: usize>(WorldData<N>);

impl<const N: usize> Display for PartialWorldData<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PARTIAL(pos = {:?}, dir = {:?})\n{}",
            self.0.mouse.pos, self.0.mouse.dir, self.0.map
        )
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
        Self(WorldData {
            map: partial_map.0,
            mouse: mouse_transform,
        })
    }

    pub fn map(&self) -> PartialMap<N> {
        PartialMap(self.map)
    }
}

pub struct CommandExecution<const N: usize> {
    pub world: WorldData<N>,
    pub command: Command,
    pub next_step: usize,
}

pub enum CommandStepResult<const N: usize> {
    Ongoing(CommandExecution<N>),
    Finished(WorldData<N>),
}

pub const SIM_MAX_DEPTH: u8 = 4;

impl<const N: usize> CommandExecution<N> {
    pub fn new(world: WorldData<N>, command: Command) -> Self {
        Self {
            world: WorldData::default(),
            command,
            next_step: 0,
        }
    }
    pub fn next(mut self) -> (Vec<Measurement>, CommandStepResult<N>) {
        let mut measurements = vec![];

        for interrupt in self.command.interrupts.iter() {
            if interrupt.at_step.matches(self.next_step) {
                let measurement = self.world.measure(interrupt.direction, SIM_MAX_DEPTH);
                measurements.push(measurement);
                if self
                    .world
                    .is_interrupt_triggered(*interrupt, self.next_step as u32)
                {
                    return (measurements, CommandStepResult::Finished(self.world));
                }
            }
        }

        self.world.mouse = self
            .world
            .mouse
            .step_once(self.command.ty)
            .expect("Command Execution outside bounds");

        self.next_step += 1;

        if self.next_step > self.command.ty.max_step_count() {
            (measurements, CommandStepResult::Finished(self.world))
        } else {
            (measurements, CommandStepResult::Ongoing(self))
        }
    }
}



