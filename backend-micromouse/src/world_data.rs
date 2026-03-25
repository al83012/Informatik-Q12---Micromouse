use crate::{map::Map, position::MouseTransform};




pub struct WorldData<const N: usize> {
    pub map: Map<N>,
    pub mouse: MouseTransform,
}





