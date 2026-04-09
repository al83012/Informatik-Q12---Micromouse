use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use console::Style;
use tracing::{debug, info};

use crate::{
    comm::micromouse_message::{
        Command, InterruptAction, MeasurementInterrupt, MovementType, StepNum, TransformedCommand,
        TransformedMovement,
    },
    transform::direction::{Direction, RelativeDirection},
    map::map::{self, Map, PartialMap, WallDiscoveryStatus},
    utils::map_display::{MapDisplay, MapDisplayWrite},
    map::measurement::{self, Measurement},
    transform::position::MouseTransform,
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
        let start_transform = self.mouse;
        let measure_dir = relative_direction.transform_by(&start_transform.dir);
        let ray_transform = MouseTransform {
            pos: start_transform.pos,
            dir: measure_dir,
        };
        debug!(target: "map/measure", "Starting measure {start_transform:?} -> {relative_direction}");
        for i in 0..=max_depth {
            let current_pos = ray_transform.moved(i);
            debug!(target: "map/measure", "Checking pos = {current_pos:?}");
            if current_pos.is_none() {
                debug!(target: "map/measure", "CHECK out of bounds --> Mark as collision");
                return Measurement {
                    value: measurement::MeasurementValue::Value { cells: i as u32 },
                    direction: measure_dir,
                    position: ray_transform.pos,
                };
            }
            let current_pos = current_pos.unwrap();
            let next_wall = self.map.wall(&current_pos.pos, &measure_dir);
            // .expect("Already checked");
            if next_wall.is_none() {
                debug!(target: "map/measure", "HIT map wall");
                return Measurement {
                    value: measurement::MeasurementValue::Value { cells: i as u32 },
                    direction: measure_dir,
                    position: ray_transform.pos,
                };
            }
            let next_wall = next_wall.unwrap();
            if i != max_depth {
                // Not yet the end --> could continue
                match next_wall {
                    //INFO: The ray only doesn't hit a wall if it is explicitly not there
                    //Does not work, if it is the max-depth: int that case HAS to create a measurement
                    WallDiscoveryStatus::Exists(false) | WallDiscoveryStatus::Visited => continue,
                    WallDiscoveryStatus::Exists(true) => {
                        return Measurement {
                            value: measurement::MeasurementValue::Value { cells: i as u32 },
                            direction: measure_dir,
                            position: ray_transform.pos,
                        };
                    }
                    WallDiscoveryStatus::Undiscovered => {
                        return Measurement {
                            value: measurement::MeasurementValue::OutsideRange {
                                at_least_cells: i as u32,
                            },
                            direction: measure_dir,
                            position: ray_transform.pos,
                        };
                    }
                }
            } else {
                match next_wall {
                    //
                    WallDiscoveryStatus::Exists(false) | WallDiscoveryStatus::Visited => {
                        return Measurement {
                            value: measurement::MeasurementValue::OutsideRange {
                                at_least_cells: i as u32,
                            },
                            direction: measure_dir,
                            position: ray_transform.pos,
                        };
                    }
                    WallDiscoveryStatus::Exists(true) => {
                        return Measurement {
                            value: measurement::MeasurementValue::Value { cells: i as u32 },
                            direction: measure_dir,
                            position: ray_transform.pos,
                        };
                    }
                    WallDiscoveryStatus::Undiscovered => {
                        return Measurement {
                            value: measurement::MeasurementValue::OutsideRange {
                                at_least_cells: i as u32,
                            },
                            direction: measure_dir,
                            position: ray_transform.pos,
                        };
                    }
                }
            }
        }
        Measurement {
            value: measurement::MeasurementValue::OutsideRange {
                at_least_cells: max_depth as u32,
            },
            direction: measure_dir,
            position: ray_transform.pos,
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
                WallDiscoveryStatus::Exists(false) | WallDiscoveryStatus::Visited,
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
            "{}",
            self.0 // "PARTIAL(pos = {:?}, dir = {:?})\n{}",
                   // self.0.mouse.pos, self.0.mouse.dir, self.0.map
        )
    }
}

impl<const N: usize> Deref for PartialWorldData<N> {
    type Target = WorldData<N>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const N: usize> DerefMut for PartialWorldData<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
    pub fn with_interrupt_termination_triggered(
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
            (
                WallDiscoveryStatus::Exists(false) | WallDiscoveryStatus::Visited,
                InterruptAction::StopIfOpen,
            ) => {
                if should_trigger {
                    *deciding_wall
                } else {
                    return None;
                }
            }

            // Interrupt will never be triggered
            (
                WallDiscoveryStatus::Exists(false) | WallDiscoveryStatus::Visited,
                InterruptAction::StopIfBlocked,
            ) => {
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

impl<const N: usize> Default for PartialWorldData<N> {
    fn default() -> Self {
        Self(WorldData::default())
    }
}





pub struct CommandExecution<const N: usize> {
    pub world: WorldData<N>,
    pub command: Command,
    pub next_step: usize,
}

// pub enum CommandStepResult<const N: usize> {
//     Ongoing(CommandExecution<N>),
//     Finished(WorldData<N>),
// }

pub enum EndState<const N: usize> {
    Ongoing(CommandExecution<N>),
    Finished(WorldData<N>),
}

pub struct CommandStepResult<const N: usize> {
    /// if nothing was done, = 0;
    pub num_of_finished_steps: usize,
    pub is_continuing: EndState<N>,
    pub measurements: Vec<Measurement>,
}

pub const SIM_MAX_DEPTH: u8 = 4;

impl<const N: usize> CommandExecution<N> {
    pub fn new(world: WorldData<N>, command: Command) -> Self {
        Self {
            world,
            command,
            next_step: 0,
        }
    }

    pub fn next(mut self) -> CommandStepResult<N> {
        let max_steps = self.command.ty.max_step_count();
        info!(target: "map", "CURRENT EXECUTION STEP {} / {}", self.next_step + 1, max_steps);

        let mut measurements = vec![];

        for interrupt in self.command.interrupts.iter() {
            if interrupt.at_step.matches(self.next_step) {
                debug!(target: "map/measure", "PROCESS INTERRUPT {interrupt}");
                let measurement = self.world.measure(interrupt.direction, SIM_MAX_DEPTH);
                measurements.push(measurement);
                if self
                    .world
                    .is_interrupt_triggered(*interrupt, self.next_step as u32)
                {
                    info!(target: "map/measure", "--> TRIGGERED");
                    return CommandStepResult {
                        num_of_finished_steps: self.next_step,
                        is_continuing: EndState::Finished(self.world),
                        measurements,
                    };
                }
            }
        }

        if self.next_step >= max_steps {
            // If max steps is 0, it should finish before even doing transforms
            info!(target: "map", "Max step reached ({})", max_steps);
            return CommandStepResult {
                num_of_finished_steps: self.next_step,
                is_continuing: EndState::Finished(self.world),
                measurements,
            };
        }

        debug!(target: "map", "Before step: {:?}", self.world.mouse);
        self.world.mouse = self
            .world
            .mouse
            .step_once(self.command.ty)
            .expect("Command execution outside bounds");
        debug!(target: "map", "After step: {:?}", self.world.mouse);

        self.next_step += 1;
        //Now: next_step is the number of steps completed

        CommandStepResult {
            num_of_finished_steps: self.next_step,
            is_continuing: EndState::Ongoing(self),
            measurements,
        }
    }
}

impl<const N: usize> Display for WorldData<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map_display = MapDisplay::from(&self.map);
        let mut cell = map_display
            .cell_mut(self.mouse.pos)
            .expect("Should exist in valid world");
        let mut center = cell.center();
        center.set_char(match self.mouse.dir {
            Direction::PosX => '>',
            Direction::PosY => 'v',
            Direction::NegX => '<',
            Direction::NegY => 'A',
        });
        center.apply_style(Style::new().on_red().on_bright().black());
        write!(f, "{}", map_display)
    }
}


