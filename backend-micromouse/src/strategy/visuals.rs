use std::sync::mpsc::Receiver;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Sender, UnboundedReceiver, UnboundedSender};

use derive_more::with_trait::*;

use crate::{
    comm::website::FrontendResponse,
    strategy::strategy_tree::{AbsoluteLayerId, AbsoluteNodeId, AbsolutePathId},
    transform::position::MouseTransform,
    utils::path::Path,
};

#[derive(Debug, From, Serialize, Deserialize)]
pub enum TreeVisualEvent {
    #[from]
    PathVisualEvent(PathVisualEvent),
    #[from]
    CmdVisualEvent(CmdVisualEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmdVisualEvent {
    layer_id: AbsoluteLayerId,
    ty: CmdVisualEventType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CmdVisualEventType {
    Send,
    Finish,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathVisualEvent {
    path_id: AbsolutePathId,
    ty: PathVisualEventType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PathVisualEventType {
    Create {
        path: PathSegment,
    },
    Remove,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathSegment {
    from: MouseTransform,
    to: MouseTransform,
}

pub struct FrontendVisuals {
    event_sender: UnboundedSender<TreeVisualEvent>,
}

impl FrontendVisuals {
    pub async fn visual_event_channel() -> (Self, UnboundedReceiver<TreeVisualEvent>) {
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
        (Self { event_sender: send }, recv)
    }
    pub fn create_path(
        &mut self,
        path_id: AbsolutePathId,
        path: PathSegment,
    ) {
        self.event_sender
            .send(TreeVisualEvent::from(PathVisualEvent {
                path_id,
                ty: PathVisualEventType::Create {
                    path,
                },
            }))
            .expect("Should be open during execution");
    }

    pub fn remove_path(&self, path_id: AbsolutePathId) {
        self.event_sender
            .send(
                PathVisualEvent {
                    path_id,
                    ty: PathVisualEventType::Remove,
                }
                .into(),
            )
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
