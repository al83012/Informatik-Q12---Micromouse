use std::{collections::HashMap, fmt::Display};

use console::Style;

use crate::{
    comm::{
        micromouse_manager::MicromouseEvent,
        website::{BatchedFrontendMessage, DiscoveryMessage, FrontendMessage},
    },
    map::{map::Map, world_data::WorldData},
    strategy::{
        strategy_tree::{AbsoluteLayerId, AbsoluteNodeId, RelativeNodeId},
        visuals::{
            PathVisualEvent,
            PathVisualEventType::{self, Create},
            TreeVisualEvent,
        },
    },
    transform::{direction::Direction, position::MouseTransform},
    utils::{
        map_display::{self, MapDisplay, MapDisplayWrite},
        path::{Path, PathReference},
    },
};

pub struct FrontendDisplay<const N: usize> {
    pub root: AbsoluteNodeId,
    pub nodes: HashMap<AbsoluteNodeId, FrontendDisplayNode>,
    pub world: WorldData<N>,
}

pub struct FrontendDisplayNode {
    pub children: HashMap<AbsoluteNodeId, Path>,
    pub parent: Option<AbsoluteNodeId>,
}

impl<const N: usize> FrontendDisplay<N> {
    pub fn new() -> Self {
        let first_element = FrontendDisplayNode {
            children: HashMap::new(),
            parent: None,
        };
        let first_element_id = AbsoluteNodeId {
            layer_id: AbsoluteLayerId(0),
            node_id: RelativeNodeId(0),
        };

        FrontendDisplay {
            root: first_element_id,
            nodes: HashMap::from([(first_element_id, first_element)]),
            world: WorldData::default(),
        }
    }
    pub fn update(&mut self, frontend_msg: &BatchedFrontendMessage) {
        for msg in frontend_msg.0.iter() {
            match msg {
                FrontendMessage::VisualEvent(TreeVisualEvent::PathVisualEvent(visual_event)) => {
                    self.update_visual_event(visual_event)
                }
                FrontendMessage::MicromouseEvent(MicromouseEvent::UpdatedMap(map_update)) => {
                    self.update_discovery(map_update)
                }
                FrontendMessage::MicromouseEvent(MicromouseEvent::UpdatePosition(new_pos)) => {
                    self.update_pos(new_pos)
                }
                _ => {}
            }
        }
    }

    pub fn update_visual_event(&mut self, visual_event: &PathVisualEvent) {
        let node_id = visual_event.associated_node;

        let ty = &visual_event.ty;

        match ty {
            PathVisualEventType::Remove => {
                let removed_node = self
                    .nodes
                    .remove(&node_id)
                    .expect("Node to remove does not exist");
                assert_eq!(removed_node.parent, None);
                assert_eq!(removed_node.children.len(), 1);

                let next_root = removed_node.children.iter().next().expect("Len = 1");
                self.root = *next_root.0;
                let next_root = self.nodes.get_mut(next_root.0).expect("Child has to exist");
                next_root.parent = None;
            }
            PathVisualEventType::Create {
                path,
                leads_to_child_node,
            } => {
                let mut new_path = Path::new(path.from);
                new_path.connect_to(path.to);
                self.nodes
                    .insert(*leads_to_child_node, FrontendDisplayNode::new(node_id));
                self.nodes
                    .get_mut(&node_id)
                    .expect("Parent must exist")
                    .children
                    .insert(*leads_to_child_node, new_path);
            }
            PathVisualEventType::Prune => {
                let node = self.nodes.get(&node_id).expect("Node does not exist");

                if let Some(parent) = node.parent {
                    let parent_node = self.nodes.get_mut(&parent).expect("Parent has to exist");
                    parent_node.children.remove(&node_id);
                }

                let mut removal_list = vec![node_id];
                while let Some(remove_next) = removal_list.pop() {
                    let removed = self
                        .nodes
                        .remove(&remove_next)
                        .expect("Removal without being added");
                    let new_removals = removed.children.keys();
                    removal_list.append(&mut new_removals.cloned().collect());
                }
            }
        }
    }
    pub fn update_discovery(&mut self, map_update: &DiscoveryMessage) {
        for cell_discovery in map_update.cell_discoveries.iter() {
            let pos = cell_discovery.at_cell;
            let new_val = cell_discovery.new_status;

            *self.world.map.cell_mut(&pos).expect("Should be in bounds") = new_val;
        }

        for wall_discovery in map_update.wall_discoveries.iter() {
            let pos = wall_discovery.from_cell;
            let dir = wall_discovery.in_direction;
            let new_val = wall_discovery.new_status;

            *self
                .world
                .map
                .wall_mut(&pos, &dir)
                .expect("Should be in bounds") = new_val;
        }
    }
    pub fn update_pos(&mut self, new_pos: &MouseTransform) {
        self.world.mouse = *new_pos;
    }
}

impl FrontendDisplayNode {
    pub fn new(from_parent: AbsoluteNodeId) -> Self {
        Self {
            children: HashMap::new(),
            parent: Some(from_parent),
        }
    }
}

impl<const N: usize> Display for FrontendDisplay<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map_display = MapDisplay::from(&self.world.map);

        let root = self.root;

        let mut next_draw_layer = vec![root];

        while !next_draw_layer.is_empty() {
            let mut draw_layer_after = vec![];
            for node_id in next_draw_layer.iter() {
                let node = self.nodes.get(node_id).expect("Node should exist");

                for (child_id, child_path) in node.children.iter() {
                    draw_layer_after.push(*child_id);

                    let mut path_ref = PathReference::new(child_path, &mut map_display);
                    path_ref.set_char('*');
                }
            }
            next_draw_layer = draw_layer_after;
        }

        let dir_char = match self.world.mouse.dir {
            Direction::PosX => '>',
            Direction::PosY => 'V',
            Direction::NegX => '<',
            Direction::NegY => 'A',
        };

        let mut cell_ref = map_display
            .cell_mut(self.world.mouse.pos)
            .expect("Cell should exist");
        let mut cell_ref = cell_ref.center();
        cell_ref.set_char(dir_char);
        cell_ref.apply_style(Style::new().on_red());

        write!(f, "{map_display}")
    }
}
