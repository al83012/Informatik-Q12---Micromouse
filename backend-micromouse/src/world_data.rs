use std::{fmt::Display, ops::Deref};

use crate::{
    comm::micromouse_message::InterruptAction,
    direction::RelativeDirection,
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
        self.map
            .wall(
                &self.mouse.pos,
                &relative_direction.transform_by(&self.mouse.dir),
            )
            .expect("")
    }

}



pub struct TransformingWorldData<const N: usize> {
    // world_data: 
}



/// Same as WorldData, but signifies, that it is not the problem state at the end of a step, but
/// an incomplete look into the future (The contained map does not include all the information that
/// should be available at the position and rotation of the mouse, as this rotation and position is
/// yet to be reached)
#[derive(Clone)]
pub struct PartialWorldData<const N: usize>(WorldData<N>);

impl<const N: usize> Display for PartialWorldData<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PARTIAL(pos = {:?}, dir = {:?})\n{}", self.0.mouse.pos, self.0.mouse.dir, self.0.map)
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
        Self(WorldData { map: partial_map.0, mouse: mouse_transform })
    }

    pub fn map(&self) -> PartialMap<N> {
        PartialMap(self.map)
    }
}
