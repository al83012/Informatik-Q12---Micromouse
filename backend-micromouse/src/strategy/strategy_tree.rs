use std::{
    collections::{hash_map, HashMap},
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Add, Sub},
};

use crate::{
    comm::micromouse_message::{Command, CommandMessage},
    map::{
        command_world_state::{
            CommandOutcomes, CommandTerminationReason, FilteredCommandApplication,
            PathLocalInterruptId, PathLocalOutcomeId,
        },
        map::{Map, PartialMap},
        world_data::{PartialWorldData, WorldData},
    },
    strategy::strategy::{
        ComputedActions, GoalPosition, Strategy, StrategyComputationResult, StrategyError,
    },
};

pub struct StrategyTree<const N: usize, S: Strategy<N> + Clone> {
    config: StrategyTreeConfig<N, S>,
    /// layers[0] = layer the micromouse is currently processing
    layers: Vec<StrategyTreeLayer<N, S>>,
    highest_sent_layer: usize,
    highest_eq_layer: usize,
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

pub enum StrategyTreeError {
    MultipleSuccessors,
    NoSuccessor,
}

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

type NodeActionResult<const N: usize> = Result<NodeAction<N>, StrategyError>;

pub struct NodeAction<const N: usize> {
    pub command: FilteredCommandApplication<N>,
    pub potential_outcomes: HashMap<PathLocalOutcomeId, AbsoluteNodeId>,
}

pub enum NodeExpansionResult {
    NotExpandable,
    NotYetExpandable,
    AlreadyExpanded,
    StrategyError(StrategyError),
    Expanded(usize),
}

impl<const N: usize, S> StrategyTree<N, S>
where
    // For use in the Strategy-Tree, we need to be able to duplicate the StrategyState to branch
    S: Strategy<N> + Clone,
{
    pub fn new_continuing_after(
        continue_after_doing: SentUnfinishedCommands<N>,
        tree_config: StrategyTreeConfig<N, S>,
        goal_position: GoalPosition,
    ) -> Self {
        let mut continue_after_doing = continue_after_doing.layers;
        let queue_len = continue_after_doing.len();

        // Either the first layer id is the first one of the unfinished cmds, or it is 0
        let first_layer_absolute_id = continue_after_doing
            .get(0)
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
        let highest_sent_layer = queue_len;
        let highest_eq_layer = queue_len;

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
    fn expand_fully(&mut self) -> usize {
        let node_budget = self.config.max_nodes.saturating_sub(self.node_count);
        let layer_budget = self
            .config
            .desired_depth
            .saturating_sub(self.fully_expanded_layer_count());

        let mut nodes_created = 0;
        let mut layers_fully_expanded = 0;

        'expansion: loop {
            if node_budget <= nodes_created || layer_budget <= layers_fully_expanded {
                // Out of budget
                break;
            }
            let layer_to_expand = self.lowest_expandable_layer();
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
            let mut skipped = false;
            'layer_expansion: for non_expanded_node_id in non_expanded_node_ids {
                let abs_node_id = AbsoluteNodeId {
                    layer_id: layer_to_expand,
                    node_id: non_expanded_node_id,
                };

                todo!("Handle different outputs; only count up if necc, prop errors");
                // nodes_created += self.try_expand_node(abs_node_id);

                if node_budget <= nodes_created {
                    break 'expansion;
                }
            }
            self.layer_mut(layer_to_expand)
                .expect("ID should be in bounds")
                .is_fully_expanded = true;
            layers_fully_expanded += 1;
        }
        nodes_created
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
                return NodeExpansionResult::StrategyError(e);
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
        let layer = if let Some(mut layer) = self.layer_mut(to_layer) {
            layer
        } else {
            self.fill_layers_to_id(to_layer);
            self.layer_mut(to_layer).expect("Tried to create it")
        };

        let rel_id = layer.add_node(node);

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
        self.layers
            .iter()
            .map(|l| if l.is_fully_expanded { 1 } else { 0 })
            .sum()
    }

    fn lowest_expandable_layer(&self) -> AbsoluteLayerId {
        self.layers
            .iter().find(|l| !l.is_fully_expanded)
            .map(|l| l.absolute_layer_id)
            .expect("There should always be a layer that has not yet been fully expanded (as expansion creates new non-expanded layers)")
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

    // fn expand(&mut self, node_id: )

    pub fn current_finished() -> Result<(), StrategyTreeError> {
        todo!("Remove the root (as it is done processing) and make its 1 successor the new root; Return an error if there are more or 0");
    }

    pub fn prune_with_filter(&mut self, filter: Map<N>) // -> ?
    {
        todo!("Go through all the nodes and remove those whose basis is not potentially_eq to the given filter;
Also: return all the sub-options that were pruned (and their relationship) for the frontend;
Also: regow
");
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
            let mut new_nodes = self.delete_node(node_to_delete).unwrap_or(vec![]);
            to_delete.append(&mut new_nodes);
        }
        Ok(())
    }

    fn delete_node(&mut self, node: AbsoluteNodeId) -> Result<Vec<AbsoluteNodeId>, PruneError> {
        let Some(layer) = self.layer_mut(node.layer_id) else {
            return Err(PruneError::UnknownNode(node));
        };
        layer.delete_node(node.node_id)
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
}

pub struct RelativeNodeIdCounter(pub usize);

impl RelativeNodeIdCounter {
    pub fn next(&mut self) -> RelativeNodeId {
        let id = RelativeNodeId(self.0);
        self.0 += 1;
        id
    }
}

impl Default for RelativeNodeIdCounter {
    fn default() -> Self {
        Self(0)
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
}
