use crate::{
    comm::micromouse_message::InterruptAction,
    direction::RelativeDirection,
    map::{self, Map, WallDiscoveryStatus},
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
        self.map.apply_measurement(&measurement)
    }
    pub fn measure(&self, relative_direction: RelativeDirection) -> &WallDiscoveryStatus {
        self.map
            .wall(
                &self.mouse.pos,
                &relative_direction.transform_by(&self.mouse.dir),
            )
            .expect("")
    }

}
