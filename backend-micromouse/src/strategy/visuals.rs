use std::sync::mpsc::Receiver;

use tokio::sync::mpsc::Sender;

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
    ty: PathVisualEventType,
}

pub type TreeDepth = usize;

pub enum PathVisualEventType {
    Create {
        before_child: AbsoluteNodeId,
        path: PathSegment,
        depth: TreeDepth,
    },
    UpdateDepth {
        depth: TreeDepth,
    },
    Remove,
}

pub struct PathSegment {
    from: MouseTransform,
    to: MouseTransform,
}

pub struct FrontendVisuals {
    event_sender: Sender<PathVisualEvent>,
}

impl FrontendVisuals {
    pub async fn visual_event_channel() -> (Self, Receiver<PathVisualEvent>) {
        todo!()
    }
    pub async fn create_path(
        &mut self,
        path_id: AbsolutePathId,
        to_child: AbsoluteNodeId,
        at_depth: TreeDepth,
        path: PathSegment,
    ) {
        self.event_sender
            .send(PathVisualEvent {
                path_id,
                ty: PathVisualEventType::Create {
                    before_child: to_child,
                    path,
                    depth: at_depth,
                },
            })
            .await
            .expect("Should be open during execution");
    }

    pub async fn remove_path(&self, path_id: AbsolutePathId) {
        self.event_sender
            .send(PathVisualEvent {
                path_id,
                ty: PathVisualEventType::Remove,
            })
            .await
            .expect("Should be open during execution");
    }

    pub async fn update_depth(&self, path_id: AbsolutePathId, depth: TreeDepth) {
        self.event_sender
            .send(PathVisualEvent {
                path_id,
                ty: PathVisualEventType::UpdateDepth { depth },
            })
            .await
            .expect("Should be open during execution");
    }
}

impl PathSegment {
    pub fn new(from: MouseTransform, to: MouseTransform) -> Option<Self> {
        let p = PathSegment { from, to };
        if from.pos == to.pos {
            return Some(p);
        }
        if from.dir == to.dir && (from.pos.x == to.pos.x || from.pos.y == to.pos.y) {
            return Some(p);
        }
        None
    }
}
