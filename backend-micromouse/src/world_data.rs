use crate::{map::{self, Map}, measurement::{self, Measurement}, position::MouseTransform};




pub struct WorldData<const N: usize> {
    pub map: Map<N>,
    pub mouse: MouseTransform,
}


impl<const N: usize>Default for  WorldData<N> {
    fn default() -> Self {
        Self {
            map: Map::<N>::new(),
            mouse: MouseTransform::default(),
        }
    }
}


impl<const N: usize> WorldData<N> {
    pub fn apply_measurement(&mut self, measurement: &Measurement) -> Result<crate::comm::website::DiscoveryMessage, map::MapInconsistencyError> {
        self.map.apply_measurement(&measurement)
    }
}





