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
    pub associated_node: AbsoluteNodeId,
    pub ty: PathVisualEventType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PathVisualEventType {
    Create {
        path: PathSegment,
        leads_to_child_node: AbsoluteNodeId,
    },
    Remove,
    Prune,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathSegment {
    pub from: MouseTransform,
    pub to: MouseTransform,
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
        path: PathSegment,
        from_node: AbsoluteNodeId,
        to_node: AbsoluteNodeId,
    ) {
        self.event_sender
            .send(TreeVisualEvent::from(PathVisualEvent {
                associated_node: from_node,
                ty: PathVisualEventType::Create {
                    path,
                    leads_to_child_node: to_node,
                },
            }))
            .expect("Should be open during execution");
    }

    pub fn remove_node(&self, node: AbsoluteNodeId) {
        self.event_sender
            .send(
                PathVisualEvent {
                    associated_node: node,
                    ty: PathVisualEventType::Remove,
                }
                .into(),
            )
            .expect("Should be open during execution");
    }

    pub fn prune_path(&self, from_including_node: AbsoluteNodeId) {
        self.event_sender
            .send(
                PathVisualEvent {
                    associated_node: from_including_node,
                    ty: PathVisualEventType::Prune,
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
