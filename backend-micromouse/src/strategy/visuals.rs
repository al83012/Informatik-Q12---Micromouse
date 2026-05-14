use crate::{
    comm::website::FrontendResponse,
    strategy::strategy_tree::{AbsoluteNodeId, AbsolutePathId},
    transform::position::MouseTransform,
    utils::path::Path,
};

pub enum TreeVisualEvent {
    PathVisualEvent(PathVisualEvent),
}

pub struct PathVisualEvent {
    path_id: AbsolutePathId,
}

pub type TreeDepth = usize;

pub enum PathVisualEventType {
    Create {before_child: AbsoluteNodeId, path: PathSegment, depth: TreeDepth },
    UpdateDepth { depth: TreeDepth },
    Remove,
}

pub struct PathSegment {
    from: MouseTransform,
    to: MouseTransform,
}

pub struct FrontendVisuals {}

impl FrontendVisuals {
    pub fn create_path(
        &mut self,
        path_id: AbsolutePathId,
        to_child: AbsoluteNodeId,
        at_depth: TreeDepth,
        path: PathSegment,
    ) {
        todo!()
    }

    pub fn remove_path(&mut self, path_id: AbsolutePathId) {
        todo!()
    }

    pub fn update_depth(&mut self, depth: TreeDepth) {
        todo!()
    }
}

impl PathSegment {
    pub fn new(from: MouseTransform, to: MouseTransform) -> Option<Self> {
        todo!()
    }
}
