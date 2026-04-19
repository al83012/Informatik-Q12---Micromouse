use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Add, Sub},
};

use crate::{
    comm::micromouse_message::Command,
    map::{
        check::PotentiallyEq,
        command_world_state::{FilteredCommandApplication, PathLocalOutcomeId, RejectedOutcomes},
        map::PartialMap,
        world_data::PartialWorldData,
    },
    strategy::strategy::{
        ComputedActions, GoalPosition, Strategy, StrategyComputationResult, StrategyEndState,
    },
};

pub enum StrategyTreeError {
    WhilePruning(PruneError),
    WhileExpanding(TreeExpansionError),
    MeasureDoesNotMatchInner,
}

pub struct StrategyTree<const N: usize, S: Strategy<N> + Clone> {
    config: StrategyTreeConfig<N, S>,
    /// layers[0] = layer the micromouse is currently processing
    layers: Vec<StrategyTreeLayer<N, S>>,
    highest_sent_layer: usize,
    highest_eq_layer: usize,
    highest_full_layer: usize,
    first_layer_absolute_id: AbsoluteLayerId,
    node_count: usize,
    goal_position: GoalPosition,
}

pub struct StrategyTreeLayer<const N: usize, S: Strategy<N>> {
    nodes: HashMap<RelativeNodeId, StrategyTreeNode<N, S>>,
    absolute_layer_id: AbsoluteLayerId,
    is_fully_expanded: bool,
    eq: Option<Command>,
    node_count: usize,
    node_id_counter: RelativeNodeIdCounter,
}

pub struct SentTreeLayer<const N: usize> {
    nodes: HashMap<RelativeNodeId, SentCommandNode<N>>,
    absolute_layer_id: AbsoluteLayerId,
    // is always full
    // is always merged (as it was sent)
    node_count: usize,
}

// Always counting up
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AbsoluteLayerId(pub usize);

// The lowest layer of a tree has id 0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RelativeLayerId(pub usize);
//
// pub enum StrategyTreeError {
//     MultipleSuccessors,
//     NoSuccessor,
// }

pub struct StrategyTreeConfig<const N: usize, S: Strategy<N> + Clone> {
    pub strategy_config: S::Config,
    pub desired_depth: usize,
    pub max_nodes: usize,
}

// = Layer 1..
pub struct StrategyTreeNode<const N: usize, S: Strategy<N>> {
    pub on_basis_of_world: PartialWorldData<N>,
    pub on_basis_of_state: Option<S>, // Could be none if it is a step taken over from another tree
    // or if it is a substep
    pub applied_strategy: Option<NodeActionResult<N>>, // The StrategyAction may not be known (as the
    // strategy may reject working with partial actions)
    // Once the strategy is known, all the potential_outcomes have to be inserted and the nodes
    // added
    pub as_branch_from_parent: Option<AbsolutePathId>,
}

pub struct SentCommandNode<const N: usize> {
    pub on_basis_of_world: PartialWorldData<N>,
    pub applied_strategy: NodeActionResult<N>,
}

type NodeActionResult<const N: usize> = Result<NodeAction<N>, StrategyEndState>;

pub struct NodeAction<const N: usize> {
    pub command: FilteredCommandApplication<N>,
    pub potential_outcomes: HashMap<PathLocalOutcomeId, AbsoluteNodeId>,
}

pub enum NodeExpansionResult {
    NotExpandable,
    NotYetExpandable,
    AlreadyExpanded,
    EndState(StrategyEndState),
    Expanded(usize),
}

pub struct TreeExpansionError {
    node: AbsoluteNodeId,
    expansion: NodeExpansionResult,
}

pub struct TreeExpansionSuccess {
    nodes: usize,
    layers: usize,
}

impl<const N: usize, S> StrategyTree<N, S>
where
    // For use in the Strategy-Tree, we need to be able to duplicate the StrategyState to branch
    S: Strategy<N> + Clone,
{
    pub fn new_continuing_after(
        continue_after_doing: SentUnfinishedCommands<N>,
        _tree_config: StrategyTreeConfig<N, S>,
        _goal_position: GoalPosition,
    ) -> Self {
        let continue_after_doing = continue_after_doing.layers;
        let queue_len = continue_after_doing.len();

        // Either the first layer id is the first one of the unfinished cmds, or it is 0
        let _first_layer_absolute_id = continue_after_doing
            .first()
            .map(|l| l.absolute_layer_id)
            .unwrap_or(AbsoluteLayerId(0));

        // TODO:
        // the last layer of the sent tree layers need to be pre-expanded by taking the outcomes of
        // the filtered application and already adding the nodes for that to the next layer
        // Then, the sent layers will have to be transformed to a non-expandable node of this
        // strategy (Maybe add a trait for that).
        // The newly pre-expanded ones will also have to be prepared with a "fresh" clone of the
        // Strategy (as they are basically its starting point)
        //
        // let continue_with_node_count = continue_after_doing.iter().map(|l| l.node_count).sum();

        // Those are currently the only layers which are both sent and finished
        let _highest_sent_layer = queue_len;
        let _highest_eq_layer = queue_len;

        // The last layer has to be expandable
        // if let Some(last) = continue_after_doing.last_mut() {
        //     last.
        // }

        todo!("aksdhföakshd");
        // let mut this = Self {
        //     config: tree_config,
        //     highest_sent_layer,
        //     highest_eq_layer,
        //     layers: continue_after_doing,
        //     first_layer_absolute_id,
        //     node_count: continue_with_node_count,
        //     goal_position,
        // };

        // this.expand_fully();

        //         todo!(
        //         "This constructor creates a new StrategyTree, whose first layers are filled up from ConfirmedUnprocessedCommands, such that they will not be changed (as they are already confirmed/sent)
        //         This means, that we can have a constructor, which can represent the option, that there are some commands that will finish and only then the chosen strategy can take place
        // "
        //     )
        // this
    }

    // returns the number of nodes this action creates
    fn expand_fully(&mut self) -> Result<TreeExpansionSuccess, TreeExpansionError> {
        let node_budget = self.config.max_nodes.saturating_sub(self.node_count);
        let layer_budget = self
            .config
            .desired_depth
            .saturating_sub(self.fully_expanded_layer_count());

        let mut nodes_created = 0;
        let layers_fully_expanded = 0;

        let highest_full_layer = self.highest_full_layer();

        // Once a layer in-between was not fully expanded, that layer and all layers after that
        // will not incr the fully_expanded_layer-counter
        let mut skipped_layer = false;
        // Iterating through all the layers that are still to be expanded
        'expansion: for i in 0..layer_budget {
            if node_budget <= nodes_created || layer_budget <= layers_fully_expanded {
                // Out of budget
                break;
            }

            // in step 0, it will be the lowest non-expanded layer
            let layer_to_expand = highest_full_layer + RelativeLayerId(i + 1);
            // let layer_to_expand = self.lowest_expandable_layer();
            let non_expanded_node_ids = {
                let layer = self
                    .layer_mut(layer_to_expand)
                    .expect("ID should be in bounds");

                layer
                    .nodes
                    .iter()
                    .filter(|(_k, v)| v.applied_strategy.is_none())
                    .map(|(k, _v)| *k)
                    .collect::<Vec<_>>()
            };
            'layer_expansion: for non_expanded_node_id in non_expanded_node_ids {
                let abs_node_id = AbsoluteNodeId {
                    layer_id: layer_to_expand,
                    node_id: non_expanded_node_id,
                };

                // todo!("Handle different outputs; only count up if necc, prop errors");

                let node_expansion_result = self.try_expand_node(abs_node_id);
                match node_expansion_result {
                    NodeExpansionResult::NotExpandable => {
                        return Err(TreeExpansionError {
                            node: abs_node_id,
                            expansion: node_expansion_result,
                        });
                    }
                    NodeExpansionResult::NotYetExpandable => {
                        skipped_layer = true;
                        // INFO: This should be alright, as long as the root node isn't being
                        // completed and is the only one left
                    }
                    NodeExpansionResult::AlreadyExpanded => {
                        // This shouldn't happen, we already filtered them
                        return Err(TreeExpansionError {
                            node: abs_node_id,
                            expansion: node_expansion_result,
                        });
                    }
                    NodeExpansionResult::EndState(_s) => {
                        // That is alright, we will throw the error once this part of the tree is
                        // visited
                    }
                    NodeExpansionResult::Expanded(num_of_nodes) => {
                        nodes_created += num_of_nodes;
                    }
                }

                if node_budget <= nodes_created {
                    break 'expansion;
                }
            }
            if skipped_layer {
                // We skipped some node expansion in this layer as it was not yet available to us
                // INFO: it will still try to expand the already existing layers up to that depth,
                // but it will not incr the expanden-layer-counter
            } else {
                self.highest_full_layer += 1;
                self.layer_mut(layer_to_expand)
                    .expect("ID should be in bounds")
                    .is_fully_expanded = true;
            }
        }
        Ok(TreeExpansionSuccess {
            nodes: nodes_created,
            layers: layers_fully_expanded,
        })
    }

    // Returns the number of children this operation created
    fn try_expand_node(&mut self, node_id: AbsoluteNodeId) -> NodeExpansionResult {
        let mut nodes_created = 0;
        let goal_position = self.goal_position;

        // INFO: ############# Getting the node ##############################
        let node = self
            .node_mut(node_id)
            .expect("Passed inside the tree; should be valid");
        // INFO: ############# Checking that it isn't already expanded ##############################
        if node.applied_strategy.is_some() {
            // The node already is fully expanded
            return NodeExpansionResult::AlreadyExpanded;
        }

        let basis_world = &node.on_basis_of_world;

        // INFO: ############# Checking that it has a strategy needed for expansion ##############################
        let Some(basis_strategy_state) = &node.on_basis_of_state else {
            // There is no strategy to base this expansion on
            return NodeExpansionResult::NotExpandable;
        };

        // INFO: ############# If the strategy does not return a proper value, it is just not yet ready ##############################
        let StrategyComputationResult::Computed(expansion_actions) =
            basis_strategy_state.next_cmd(basis_world, &goal_position)
        else {
            return NodeExpansionResult::NotYetExpandable;
        };

        let expansion_actions: ComputedActions<N, S> = match expansion_actions {
            Ok(o) => o,
            Err(e) => {
                // Only trigger this error if this node is reached
                node.applied_strategy = Some(Err(e.clone()));
                return NodeExpansionResult::EndState(e);
            }
        };

        // INFO: ############# First, apply on the one node ##############################
        let mut apply_on_node = vec![node_id];

        // There could be multiple actions, that were batched:
        // In that case, we have to work through all the different actions layer by layer and
        // always expand the nodes of the previous layer
        // INFO: ############# If there are multiple substeps, apply on all the children of
        // expansion ##############################
        for expansion_action in expansion_actions.0.into_inner().into_iter() {
            let do_cmd = expansion_action.after_command;
            let strategy_state_after = expansion_action.next_strategy_state;

            // At the first step there should only be one node to expand, but further down the line
            // there could be an entire collection of them
            let mut new_apply_on_node = vec![];
            for parent_node in apply_on_node {
                let child_node_layer = parent_node.layer_id + RelativeLayerId(1);
                let basis_world = {
                    let parent_node = self.node(parent_node).expect("Checked");
                    parent_node.on_basis_of_world.clone()
                };

                let cmd_application = FilteredCommandApplication::new(
                    Some(basis_world.clone().into()),
                    do_cmd.clone(),
                );

                // There will be 1 child per potential outcome of the given command
                let mut children = HashMap::new();
                for (child_path_id, child_world) in cmd_application
                    .potential_outcomes_given_filter()
                    .potential_outcomes
                {
                    let path_id = AbsolutePathId {
                        from_node: parent_node,
                        branch: child_path_id,
                    };
                    let child_node = StrategyTreeNode::new_leaf(
                        child_world.clone(),
                        strategy_state_after.clone(),
                        path_id,
                    );
                    let child_node_id = self.add_node(child_node, child_node_layer);
                    nodes_created += 1;
                    new_apply_on_node.push(child_node_id);

                    children.insert(child_path_id, child_node_id);
                }

                let action = NodeAction {
                    command: cmd_application,
                    potential_outcomes: children,
                };

                //Lastly, add the actual children-information to the parent
                let parent_node = self.node_mut(parent_node).expect("Checked");
                parent_node.applied_strategy = Some(Ok(action));
            }
            apply_on_node = new_apply_on_node;
        }

        NodeExpansionResult::Expanded(nodes_created)
    }

    fn add_node(
        &mut self,
        node: StrategyTreeNode<N, S>,
        to_layer: AbsoluteLayerId,
    ) -> AbsoluteNodeId {
        let layer = if let Some(layer) = self.layer_mut(to_layer) {
            layer
        } else {
            self.fill_layers_to_id(to_layer);
            self.layer_mut(to_layer).expect("Tried to create it")
        };

        let rel_id = layer.add_node(node);

        self.node_count += 1;

        AbsoluteNodeId {
            layer_id: to_layer,
            node_id: rel_id,
        }
    }

    fn fill_layers_to_id(&mut self, to_layer: AbsoluteLayerId) -> usize {
        let highest_layer_id = self
            .layers
            .last()
            .expect("There should always be at least one layer")
            .absolute_layer_id;

        if highest_layer_id < to_layer {
            let diff = to_layer.0 - highest_layer_id.0;
            for i in highest_layer_id.0 + 1..=to_layer.0 {
                let id = AbsoluteLayerId(i);
                self.layers.push(StrategyTreeLayer::new(id));
            }
            diff
        } else {
            0
        }
    }

    fn fully_expanded_layer_count(&self) -> usize {
        // self.layers
        //     .iter()
        //     .map(|l| if l.is_fully_expanded { 1 } else { 0 })
        //     .sum()
        self.highest_full_layer + 1
    }

    fn highest_full_layer(&self) -> AbsoluteLayerId {
        self.first_layer_absolute_id + RelativeLayerId(self.highest_full_layer)
        // self.layers
        //     .iter().find(|l| !l.is_fully_expanded)
        //     .map(|l| l.absolute_layer_id)
        //     .expect("There should always be a layer that has not yet been fully expanded (as expansion creates new non-expanded layers)")
    }

    fn node(&self, node_id: AbsoluteNodeId) -> Option<&StrategyTreeNode<N, S>> {
        let layer = self.layer(node_id.layer_id)?;
        layer.node(node_id.node_id)
    }

    fn node_mut(&mut self, node_id: AbsoluteNodeId) -> Option<&mut StrategyTreeNode<N, S>> {
        let layer = self.layer_mut(node_id.layer_id)?;
        layer.node_mut(node_id.node_id)
    }

    fn layer(&self, layer_id: AbsoluteLayerId) -> Option<&StrategyTreeLayer<N, S>> {
        let rel_id = self.relative_layer(layer_id)?;
        self.layers.get(rel_id.0)
    }

    fn layer_mut(&mut self, layer_id: AbsoluteLayerId) -> Option<&mut StrategyTreeLayer<N, S>> {
        let rel_id = self.relative_layer(layer_id)?;
        self.layers.get_mut(rel_id.0)
    }

    fn valid_layer_id(&self, layer_id: AbsoluteLayerId) -> bool {
        self.first_layer_absolute_id <= layer_id
    }

    fn relative_layer(&self, layer_id: AbsoluteLayerId) -> Option<RelativeLayerId> {
        if self.valid_layer_id(layer_id) {
            Some(layer_id - self.first_layer_absolute_id)
        } else {
            None
        }
    }

    pub fn delete_non_sent() {
        todo!("Delete all those nodes which are still deletable as the micromouse doesn't have them in the buffer yet")
    }

    // removes the root, making its successor the new root; if the successor was not yet expanded
    // into a command, it will do so at this point, returning its NodeAction
    pub fn finish_root(&mut self) -> Result<Option<NodeAction<N>>, FinishRootError> {
        let (_outcome_id, successor) = {
            let _root = self.root_node();
            let root = self.node_mut(self.root_node()).expect("Checked");

            let Some(strategy_res) = &root.applied_strategy else {
                // The root will be expanded, at least when it is placed into the root-slot (since the
                // execution starts there and the potential outcomes have to be known)
                return Err(FinishRootError::RootNotExpanded);
            };

            let node_action = match strategy_res {
                Ok(n) => n,
                Err(e) => {
                    // Per definition, the command that just finished cannot be a command which was
                    // not executable
                    return Err(FinishRootError::ImpossibleRootAction(e.clone()));
                }
            };

            let potential_outcomes = &node_action.potential_outcomes;
            let next_action_len = potential_outcomes.len();

            match next_action_len {
                0 => return Err(FinishRootError::NoSuccessor),
                1 => {
                    let (o, c) = potential_outcomes.iter().next().unwrap();
                    (*o, *c)
                }
                2.. => {
                    return Err(FinishRootError::MultipleSuccessors(
                        potential_outcomes.clone(),
                    ))
                }
            }
        };

        let next_root_expansion_res = self.try_expand_node(successor);
        match next_root_expansion_res {
            NodeExpansionResult::AlreadyExpanded => {
                // Best case
            }
            NodeExpansionResult::NotExpandable => {
                return Err(FinishRootError::SuccessorNotExpandable);
            }
            NodeExpansionResult::NotYetExpandable => {
                return Err(FinishRootError::SuccessorNotYetExpandable);
            }
            NodeExpansionResult::EndState(s) => {
                // We know that this is expanding a full layer, since the new root layer only
                // contains the successor
                self.highest_full_layer += 1;
                self.highest_eq_layer += 1; // Returning from this function counts as sending
                self.highest_sent_layer += 1; // Returning from this function counts as sending
                return Err(FinishRootError::SuccessorIsEnd(Some(s)));
            }
            NodeExpansionResult::Expanded(_) => {
                // We know that this is expanding a full layer, since the new root layer only
                // contains the successor
                // layer is now fully expanded
                self.highest_full_layer += 1;
            }
        }

        self.layers.remove(0);
        self.node_count -= 1;

        let new_first_layer = self.layers.first().expect("Has Successor");
        self.first_layer_absolute_id = new_first_layer.absolute_layer_id;
        self.highest_full_layer -= 1;
        self.highest_eq_layer -= 1;
        self.highest_sent_layer -= 1;

        Ok(None)
    }

    pub fn prune_not_potentially_eq(&mut self, filter: &PartialMap<N>) -> Result<(), PruneError> {
        let node_indices = self
            .layers
            .iter()
            .flat_map(|l| {
                l.nodes.iter().filter_map(|(k, v)| {
                    if v.on_basis_of_world.map.potentially_eq(filter) {
                        Some(AbsoluteNodeId {
                            layer_id: l.absolute_layer_id,
                            node_id: *k,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        for node in node_indices.into_iter() {
            match self.prune_node(node) {
                Ok(_) => {}
                Err(PruneError::UnknownNode(_)) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn prune_current_command_by_rejection(
        &mut self,
        rejections: &RejectedOutcomes,
    ) -> Result<(), PruneError> {
        for rejected in rejections.rejected_outcome_ids.iter() {
            let path_id = AbsolutePathId {
                from_node: self.root_node(),
                branch: *rejected,
            };
            self.prune_branch(path_id)?
        }
        Ok(())
    }

    fn root_node(&self) -> AbsoluteNodeId {
        let root_layer = self
            .layer(self.first_layer_absolute_id)
            .expect("Should always be kept valid");
        assert_eq!(root_layer.node_count, 1);
        let rel_id = root_layer.nodes.keys().next().expect("Checked");
        AbsoluteNodeId {
            layer_id: self.first_layer_absolute_id,
            node_id: *rel_id,
        }
    }

    fn prune_node(&mut self, node_and_children: AbsoluteNodeId) -> Result<(), PruneError> {
        let Some(node_to_delete) = self.node(node_and_children) else {
            return Err(PruneError::UnknownNode(node_and_children));
        };
        let Some(branch) = &node_to_delete.as_branch_from_parent else {
            return Err(PruneError::CannotDeleteRoot(node_and_children));
        };

        self.prune_branch(branch.clone())
    }

    fn prune_branch(&mut self, branch: AbsolutePathId) -> Result<(), PruneError> {
        let branch_source_id = branch.from_node;
        let Some(branch_source) = self.node_mut(branch_source_id) else {
            return Err(PruneError::UnknownNode(branch_source_id));
        };
        let Some(Ok(children)) = &mut branch_source.applied_strategy else {
            return Err(PruneError::SourceHasNoChildren(branch_source_id));
        };

        let Some(first_delete_node) = children.potential_outcomes.remove(&branch.branch) else {
            return Err(PruneError::SourceDoesNotHaveThisChild(branch));
        };

        let mut to_delete = vec![first_delete_node];

        while let Some(node_to_delete) = to_delete.pop() {
            let mut new_nodes =
                unsafe { self.delete_node_no_clean(node_to_delete).unwrap_or(vec![]) };
            to_delete.append(&mut new_nodes);
        }
        Ok(())
    }

    // Returns the nodes children
    unsafe fn delete_node_no_clean(
        &mut self,
        node: AbsoluteNodeId,
    ) -> Result<Vec<AbsoluteNodeId>, PruneError> {
        let Some(layer) = self.layer_mut(node.layer_id) else {
            return Err(PruneError::UnknownNode(node));
        };
        let l = layer.delete_node(node.node_id);
        if l.is_ok() {
            self.node_count -= 1;
        }
        l
    }

    fn update_equal_layers(&mut self) {
        let highest_full_layer = self.highest_full_layer;
        let highest_eq_layer = self.highest_eq_layer;

        for layer_offset in highest_eq_layer + 1..=highest_full_layer {
            let layer_id = self.first_layer_absolute_id + RelativeLayerId(layer_offset);
            if !self
                .layer_mut(layer_id)
                .expect("Should always be in bounds")
                .update_eq_command()
            {
                break;
            }
            self.highest_eq_layer += 1;
        }
    }

    fn new_sends(&mut self) -> Vec<Command> {
        if self.highest_sent_layer < self.highest_eq_layer {
            return self.layers[self.highest_sent_layer + 1..=self.highest_eq_layer]
                .iter()
                .map(|l| l.eq.as_ref().expect("Explicitly should exist").clone())
                .collect();
        }
        vec![]
    }

    fn merge(&mut self) // -> ?
    {
        // Try merging non-eq-layers
        todo!("Nice-to-have, but really complex")
    }

    fn transform_command(
        &mut self,
        from_node: AbsoluteNodeId,
        split_commands: (Command, Option<Command>),
    ) // ->?
    {
        let children = self
            .node(from_node)
            .expect("Internal should exist")
            .children()
            .expect("A command that we transform has to have been expanded");

        // takes in a node which has a command (otherwise, what are we even merging / merges on
        // incomplete layers are not allowed)
        //
        // instead of that 1 node leading to a child node, insert a command (if needed), which
        // would also mean pushing all the children back a layer
        //
        // The given node should not be in a layer that has already achieved eq, since that is
        // nonsensical
        // It should not however change 'fully expanded' of any layer
        //
        // If there was a strategy-state associated with the from_node, this state will either stay
        // there (if the 2nd command is none) or move back 1
        todo!("WE NEED TO DETERMINE, WHICH OUTCOMES MATCH THE ORIGINAL OUTCOMES; WHICH IS QUITE HARD -> NEED TO MATCH UP THE PARTIAL WORLDS");
    }
    unsafe fn move_node_back(&mut self, node_id: AbsoluteNodeId)
    // --> The new node_id; the id of its
    // parent;
    {
        let node = self.take_node_unclean(node_id);
        let node = node.expect("Node should exist; node_id comes from Internal");

        let parent = node.as_branch_from_parent.clone();
        let children = &node
            .applied_strategy
            .as_ref()
            .and_then(|a| a.as_ref().map(|a| a.potential_outcomes.clone()).ok());

        let new_node_id = self.add_node(node, node_id.layer_id + RelativeLayerId(1));

        if let Some(path_from_parent) = parent {
            let parent_node = self
                .node_mut(path_from_parent.from_node)
                .expect("Parent should exist");

            let children = parent_node
                .applied_strategy
                .as_mut()
                .and_then(|a| a.as_mut().map(|a| &mut a.potential_outcomes).ok())
                .expect("Just handled its child, parent should have children");

            // WARN: The real "unsafe" bit: The connection now spans 2 layers
            children.insert(path_from_parent.branch, new_node_id);
        }

        if let Some(children) = children {
            for (child_path, child_node_id) in children.iter() {
                let child_node = self
                    .node_mut(*child_node_id)
                    .expect("If it does not exist, why was its link still there");
                child_node.as_branch_from_parent = Some(AbsolutePathId {
                    from_node: new_node_id,
                    branch: *child_path,
                })
            }
        }
    }

    // Remove the node from the tree and return it (without cleaning up the connections)
    unsafe fn take_node_unclean(&mut self, node: AbsoluteNodeId) -> Option<StrategyTreeNode<N, S>> {
        self.layer_mut(node.layer_id)?.nodes.remove(&node.node_id)
    }

    // Filters the tree using the best current knowledge about the map
    // Returns any commands that have become certain by doing that
    pub fn handle_map_update(
        &mut self,
        map: &PartialMap<N>,
    ) -> Result<Vec<Command>, StrategyTreeError> {
        if self
            .node(self.root_node())
            .expect("Index should be valid")
            .on_basis_of_world
            .map
            .potentially_eq(map)
        {
            self.prune_not_potentially_eq(map)?;
            let _ = self.expand_fully()?;
            // TODO: self.merge()?;

            self.update_equal_layers();
            Ok(self.new_sends())
        } else {
            Err(StrategyTreeError::MeasureDoesNotMatchInner)
        }
    }

    // Filters the tree using the rejection-events from the currently processed command
    pub fn handle_cmd_rejection(
        &mut self,
        rejections: &RejectedOutcomes,
    ) -> Result<Vec<Command>, StrategyTreeError> {
        self.prune_current_command_by_rejection(rejections)?;
        let _ = self.expand_fully()?;
        self.update_equal_layers();
        Ok(self.new_sends())
    }

    pub fn close(&mut self) -> SentUnfinishedCommands<N> {
        todo!("The strategy tree will be closed, meaning that it will generate no new commands")
    }
}

#[derive(Clone, Debug)]
pub enum PruneError {
    UnknownNode(AbsoluteNodeId),
    CannotDeleteRoot(AbsoluteNodeId),
    SourceHasNoChildren(AbsoluteNodeId),
    SourceDoesNotHaveThisChild(AbsolutePathId),
}

#[derive(Clone, Copy, Debug)]
pub struct AbsoluteNodeId {
    layer_id: AbsoluteLayerId,
    node_id: RelativeNodeId,
}

#[derive(Clone, Debug)]
pub struct AbsolutePathId {
    from_node: AbsoluteNodeId,
    branch: PathLocalOutcomeId,
}

impl From<PruneError> for StrategyTreeError {
    fn from(value: PruneError) -> Self {
        Self::WhilePruning(value)
    }
}
impl From<TreeExpansionError> for StrategyTreeError {
    fn from(value: TreeExpansionError) -> Self {
        Self::WhileExpanding(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct RelativeNodeId(pub usize);

pub struct SentUnfinishedCommands<const N: usize> {
    // They are Strategy-Agnostic by not allowing the nodes to expand, which allows us to leave out
    // the strategy-state, which is needed for expansion
    layers: Vec<SentTreeLayer<N>>,
}

impl Sub for AbsoluteLayerId {
    type Output = RelativeLayerId;
    fn sub(self, rhs: Self) -> Self::Output {
        RelativeLayerId(self.0 - rhs.0)
    }
}

impl Add<RelativeLayerId> for AbsoluteLayerId {
    type Output = AbsoluteLayerId;
    fn add(self, rhs: RelativeLayerId) -> Self::Output {
        AbsoluteLayerId(self.0 + rhs.0)
    }
}

impl<const N: usize, S: Strategy<N>> StrategyTreeLayer<N, S> {
    pub fn node(&self, node_id: RelativeNodeId) -> Option<&StrategyTreeNode<N, S>> {
        self.nodes.get(&node_id)
    }
    pub fn node_mut(&mut self, node_id: RelativeNodeId) -> Option<&mut StrategyTreeNode<N, S>> {
        self.nodes.get_mut(&node_id)
    }
    pub fn new(absolute_layer_id: AbsoluteLayerId) -> Self {
        Self {
            node_id_counter: RelativeNodeIdCounter::default(),
            nodes: HashMap::new(),
            absolute_layer_id,
            is_fully_expanded: false,
            eq: None,
            node_count: 0,
        }
    }

    pub fn add_node(&mut self, node: StrategyTreeNode<N, S>) -> RelativeNodeId {
        let next_id = self.node_id_counter.next();
        self.nodes.insert(next_id, node);
        self.node_count += 1;
        next_id
    }

    pub fn delete_node(&mut self, node: RelativeNodeId) -> Result<Vec<AbsoluteNodeId>, PruneError> {
        let Some(node) = self.nodes.remove(&node) else {
            return Err(PruneError::UnknownNode(AbsoluteNodeId {
                layer_id: self.absolute_layer_id,
                node_id: node,
            }));
        };
        self.node_count -= 1;
        let Some(Ok(c)) = node.applied_strategy else {
            return Ok(vec![]);
        };
        Ok(c.potential_outcomes.values().cloned().collect())
    }

    pub fn equal_command(&self) -> Option<Command> {
        if !self.is_fully_expanded {
            return None;
        }
        if self.node_count == 0 {
            return None;
        }
        let mut nodes = self.nodes.iter();
        let (_, first) = nodes.next().expect("Checked");
        let Some(Ok(a)) = &first.applied_strategy else {
            return None;
        };
        let cmd = a.command.command().clone();

        for (_, other) in nodes {
            let Some(Ok(a)) = &other.applied_strategy else {
                return None;
            };
            if cmd != *a.command.command() {
                return None;
            }
        }

        Some(cmd)
    }

    // Returns true, if the layer has a common command
    pub fn update_eq_command(&mut self) -> bool {
        if self.eq.is_some() {
            return true;
        }
        if self.is_fully_expanded {
            self.eq = self.equal_command();
            if self.eq.is_some() {
                return true;
            }
        }
        false
    }
}

#[derive(Default)]
pub struct RelativeNodeIdCounter(pub usize);

pub enum FinishRootError {
    NoSuccessor,
    MultipleSuccessors(HashMap<PathLocalOutcomeId, AbsoluteNodeId>),
    RootNotExpanded,
    ImpossibleRootAction(StrategyEndState),
    SuccessorNotExpandable,
    SuccessorNotYetExpandable,
    // Either a valid end or a strategy error
    SuccessorIsEnd(Option<StrategyEndState>),
}

impl RelativeNodeIdCounter {
    pub fn next(&mut self) -> RelativeNodeId {
        let id = RelativeNodeId(self.0);
        self.0 += 1;
        id
    }
}

impl<const N: usize> SentCommandNode<N> {
    pub fn as_nonexpandable<S: Strategy<N>>(self, at: AbsolutePathId) -> StrategyTreeNode<N, S> {
        StrategyTreeNode {
            on_basis_of_world: self.on_basis_of_world,
            on_basis_of_state: None,
            applied_strategy: Some(self.applied_strategy),
            as_branch_from_parent: Some(at),
        }
    }
}

impl<const N: usize, S: Strategy<N>> StrategyTreeNode<N, S> {
    pub fn new_leaf(
        world_at_start_of_node: PartialWorldData<N>,
        state_at_start_of_node: Option<S>,
        at: AbsolutePathId,
    ) -> Self {
        Self {
            on_basis_of_world: world_at_start_of_node,
            on_basis_of_state: state_at_start_of_node,
            applied_strategy: None,
            as_branch_from_parent: Some(at),
        }
    }

    pub fn children(&self) -> Option<&HashMap<PathLocalOutcomeId, AbsoluteNodeId>> {
        self.applied_strategy
            .as_ref()
            .and_then(|a| a.as_ref().ok())
            .map(|a| &a.potential_outcomes)
    }
    pub fn children_mut(&mut self) -> Option<&mut HashMap<PathLocalOutcomeId, AbsoluteNodeId>> {
        self.applied_strategy
            .as_mut()
            .and_then(|a| a.as_mut().ok())
            .map(|a| &mut a.potential_outcomes)
    }
}
