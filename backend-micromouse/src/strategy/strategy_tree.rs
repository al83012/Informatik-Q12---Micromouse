use std::{
    collections::HashMap,
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
    merged: Option<Command>,
    node_count: usize,
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
        // strategy (Maybe add a trait for that)
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
        let goal_position = self.goal_position.clone();
        let node = self
            .node_mut(node_id)
            .expect("Passed inside the tree; should be valid");
        if node.applied_strategy.is_some() {
            // The node already is fully expanded
            return NodeExpansionResult::AlreadyExpanded;
        }

        let basis_world = &node.on_basis_of_world;

        let Some(basis_strategy_state) = &node.on_basis_of_state else {
            // There is no strategy to base this expansion on
            return NodeExpansionResult::NotExpandable;
        };

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

        for expansion_action in (*expansion_actions).iter() {}

        todo!()
    }

    fn fully_expanded_layer_count(&self) -> usize {
        self.layers
            .iter()
            .map(|l| if l.is_fully_expanded { 1 } else { 0 })
            .sum()
    }

    fn lowest_expandable_layer(&self) -> AbsoluteLayerId {
        self.layers
            .iter()
            .filter(|l| !l.is_fully_expanded)
            .next()
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

    pub fn prune_root_outcome(&mut self, outcome_id: PathLocalOutcomeId) // -->
    {
        todo!(
            "Remove the child of the root that matches the PathLocalOutcomeId;
Also: return all the sub-options that were pruned (and their relationship) for the frontend"
        );
        todo!("Also: regrow")
    }

    pub fn grow(&mut self) // -> ?
    {
        todo!("Check whether growth is within the limits of StrategyTree (desired_depth & max_nodes) & then expand those worlds that are closest to the root")
    }

    pub fn close(&mut self) -> SentUnfinishedCommands<N> {
        todo!("The strategy tree will be closed, meaning that it will generate no new commands")
    }
}

pub struct AbsoluteNodeId {
    layer_id: AbsoluteLayerId,
    node_id: RelativeNodeId,
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
}
