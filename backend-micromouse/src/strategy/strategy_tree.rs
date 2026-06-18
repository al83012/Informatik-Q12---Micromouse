use std::{
    clone,
    collections::{HashMap, VecDeque},
    hash::Hash,
    ops::{Add, Deref, Sub},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, instrument, span, Level};
use tracing_subscriber::Layer;

use crate::{
    comm::micromouse_message::Command,
    map::{
        check::PotentiallyEq,
        command_world_state::{FilteredCommandApplication, PathLocalOutcomeId, RejectedOutcomes},
        map::{Map, PartialMap},
        map_set_op::Union,
        world_data::{PartialWorldData, WorldData},
    },
    strategy::{
        strategy::{
            ComputedActions, FromConfig, GoalPosition, Strategy, StrategyComputationResult,
            StrategyEndState,
        },
        visuals::{FrontendVisuals, PathSegment},
    },
    utils::{
        hyperlink_logging::LinkFileName,
        nonempty::{NonEmpty, PotentiallyNonEmpty},
    },
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarkerLayerId {
    NotExistant,
    AtLayer(AbsoluteLayerId),
}

impl PartialOrd for MarkerLayerId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match self {
            MarkerLayerId::AtLayer(l_self) => match other {
                MarkerLayerId::NotExistant => std::cmp::Ordering::Greater,
                MarkerLayerId::AtLayer(l_other) => l_self.0.cmp(&l_other.0),
            },
            MarkerLayerId::NotExistant => match other {
                MarkerLayerId::NotExistant => std::cmp::Ordering::Equal,
                MarkerLayerId::AtLayer(_l) => std::cmp::Ordering::Less,
            },
        })
    }
}

impl Ord for MarkerLayerId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("Implemented")
    }
}

#[derive(Debug, Clone, Serialize, Error, Deserialize)]
pub enum StrategyTreeError {
    #[error("Error while Pruning branch ")]
    WhilePruning(#[from] PruneError),
    #[error("Error while expanding a node")]
    WhileExpanding(#[from] TreeExpansionError),
    #[error("Error while creating the tree")]
    WhileCreating(#[from] TreeCreationError),
    #[error("Error while declaring the root finished")]
    WhileFinishingCommand(#[from] FinishRootError),
    #[error("The given map_update cannot be valid for the internal map")]
    MapConflictsWithMapUpdate,
}

pub struct StrategyTree<const N: usize, S: Strategy<N> + Clone + FromConfig<N>> {
    config: StrategyTreeConfig<N, S>,
    layers: Vec<StrategyTreeLayer<N, S>>,
    highest_sent_layer: MarkerLayerId,
    highest_eq_layer: MarkerLayerId,
    highest_full_layer: MarkerLayerId,
    first_layer_absolute_id: AbsoluteLayerId,
    node_count: usize,
    goal_position: GoalPosition,
    visuals: FrontendVisuals,
}

pub struct StrategyTreeLayer<const N: usize, S: Strategy<N>> {
    nodes: HashMap<RelativeNodeId, StrategyTreeNode<N, S>>,
    absolute_layer_id: AbsoluteLayerId,
    // whether the layer before this one is fully expanded / whether no additional nodes will be
    // added to this layer
    is_full: bool,
    eq: Option<Command>,
    node_count: usize,
    node_id_counter: RelativeNodeIdCounter,
    is_sent: bool,
}

#[derive(Debug)]
pub struct SentTreeLayer<const N: usize> {
    nodes: HashMap<RelativeNodeId, SentCommandNode<N>>,
    absolute_layer_id: AbsoluteLayerId,
    // is always full
    // is always merged (as it was sent)
    node_count: usize,
    eq: Command,
}

// Always counting up
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Eq)]
pub struct AbsoluteLayerId(pub usize);

// The lowest layer of a tree has id 0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub struct RelativeLayerId(pub usize);

#[derive(Debug)]
pub struct StrategyTreeConfig<const N: usize, S: Strategy<N> + Clone + FromConfig<N>> {
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

#[derive(Debug)]
pub struct SentCommandNode<const N: usize> {
    pub on_basis_of_world: PartialWorldData<N>,
    pub applied_strategy: NodeActionResult<N>,
    as_branch_from_parent: Option<AbsolutePathId>,
}

type NodeActionResult<const N: usize> = Result<NodeAction<N>, StrategyEndState>;

#[derive(Debug)]
pub struct NodeAction<const N: usize> {
    pub command: FilteredCommandApplication<N>,
    pub potential_outcomes: HashMap<PathLocalOutcomeId, AbsoluteNodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeExpansionResult {
    NotExpandable,
    NotYetExpandable,
    AlreadyExpanded,
    EndState(StrategyEndState),
    Expanded(usize),
}

#[derive(Debug, Clone, Serialize, Error, Deserialize)]
#[error("At node {node:?} with result {expansion:?}")]
pub struct TreeExpansionError {
    node: AbsoluteNodeId,
    expansion: NodeExpansionResult,
}

#[derive(Debug)]
pub struct TreeExpansionSuccess {
    nodes: usize,
    layers: usize,
}

#[derive(Debug)]
pub enum StrategyStart<const N: usize> {
    ContinueAfterDoing {
        after_cmds: SentUnfinishedCommands<N>,
        reset_world: bool,
    },
    // Will create a root node from this world and the strategy-initializer
    DirectlyAtState(WorldData<N>),
}

#[derive(Debug, Clone, Serialize, Error, Deserialize)]
pub enum TreeCreationError {
    #[error("Strategy ended at start {0:?}")]
    StrategyError(#[from] StrategyEndState),
    #[error("Failed expanding the root")]
    ExpansionError(#[from] TreeExpansionError),
    #[error("Root was not expanded after attempt to do so")]
    RootNotExpanded,
}

pub struct TreeCreationSuccess<
    const N: usize,
    S: Strategy<N> + Clone + std::fmt::Debug + FromConfig<N>,
> {
    pub tree: StrategyTree<N, S>,
    pub origin_command: Option<Command>,
}

impl<const N: usize, S> StrategyTree<N, S>
where
    // For use in the Strategy-Tree, we need to be able to duplicate the StrategyState to branch
    S: Strategy<N> + Clone + std::fmt::Debug + FromConfig<N>,
{
    #[instrument(
        skip(starting_condition, visuals),
        name = "new StrategyTree",
        fields(
            description = "Creates new strategy tree, potentially on the basis of a number of commands that was sent and cannot be taken back"
        )
    )]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        starting_condition: StrategyStart<N>,
        tree_config: StrategyTreeConfig<N, S>,
        goal_position: GoalPosition,
        visuals: FrontendVisuals,
    ) -> Result<TreeCreationSuccess<N, S>, StrategyTreeError> {
        match starting_condition {
            StrategyStart::ContinueAfterDoing {
                after_cmds: SentUnfinishedCommands::HasQueue { layers },
                reset_world,
            } => {
                info!(target: "strat", "Continue after doing");
                // let num_of_unfinished_cmds = layers.len();
                let node_count = layers.iter().map(|l| l.node_count).sum::<usize>();
                let first_layer_absolute_id = layers
                    .first()
                    .map(|l| l.absolute_layer_id)
                    .unwrap_or(AbsoluteLayerId(0));
                info!(target: "strat", link_layer_id = first_layer_absolute_id.link(), "First layer which was sent, but not finished = {first_layer_absolute_id:?}");

                let layers = layers.into_inner().into_iter();
                let mut transformed_layers = layers
                    .map(StrategyTreeLayer::<N, S>::non_expandable_from_cleaned)
                    .collect::<Vec<_>>();

                let last_layer = transformed_layers.last_mut().expect("layers are nonempty");
                let last_layer_id = last_layer.absolute_layer_id;
                info!(target: "strat", link_layer_id = last_layer_id.link(), "Last layer which was sent, but not finished = {last_layer_id:?}");

                // WARN:
                // The last layer contained in the sent unfinished commands is a layer, which was
                // not yet sent, but is the expansion of the last sent layer; This means, that this
                // is the grafting point
                last_layer.nodes.iter_mut().for_each(|(_n, val)| {
                    let world = if reset_world {
                        &val.on_basis_of_world.only_pos()
                    } else {
                        &val.on_basis_of_world
                    };
                    val.applied_strategy = None;
                    val.on_basis_of_state =
                        Some(S::from_config(&tree_config.strategy_config, world));
                });

                let highest_sent_marker =
                    MarkerLayerId::AtLayer(last_layer_id - RelativeLayerId(1));

                let tree = Self {
                    config: tree_config,
                    layers: transformed_layers,
                    highest_sent_layer: highest_sent_marker,
                    highest_eq_layer: highest_sent_marker,
                    highest_full_layer: highest_sent_marker,
                    first_layer_absolute_id,
                    node_count,
                    goal_position,
                    visuals,
                };

                Ok(TreeCreationSuccess {
                    tree,
                    origin_command: None,
                })
            }
            x => {
                let starting_state = match x {
                    StrategyStart::ContinueAfterDoing {
                        after_cmds: SentUnfinishedCommands::HasBlockingRoot { world },
                        reset_world,
                    } => {
                        info!(target: "strat", "Continue after blocking root");
                        if reset_world {
                            world.only_pos()
                        } else {
                            world
                        }
                    }
                    StrategyStart::DirectlyAtState(world) => world,
                    _ => panic!("Covered previously"),
                };

                info!(target: "strat", "Without previous Strategy");
                let first_strategy = S::from_config(&tree_config.strategy_config, &starting_state);

                let first_layer_id = AbsoluteLayerId(0);

                let mut res = Self {
                    config: tree_config,
                    layers: vec![StrategyTreeLayer::new(AbsoluteLayerId(0))],
                    highest_sent_layer: MarkerLayerId::NotExistant,
                    highest_eq_layer: MarkerLayerId::NotExistant,
                    highest_full_layer: MarkerLayerId::AtLayer(first_layer_id),
                    first_layer_absolute_id: AbsoluteLayerId(0),
                    node_count: 0,
                    goal_position,
                    visuals,
                };

                let first_node_id = res.add_node(
                    StrategyTreeNode::new_orphan(
                        PartialWorldData::from(starting_state),
                        Some(first_strategy),
                    ),
                    AbsoluteLayerId(0),
                );

                res.layer_mut(first_layer_id).expect("Checked").is_full = true;

                res.expand_fully().map_err(TreeCreationError::from)?;

                let root_send = res
                    .node(first_node_id)
                    .expect("Just added it")
                    .applied_strategy
                    .as_ref()
                    .ok_or(TreeCreationError::RootNotExpanded);
                let root_send = root_send?;
                let root_send = root_send
                    .as_ref()
                    .map_err(StrategyEndState::clone)
                    .map_err(TreeCreationError::from)?;
                let first_cmd = root_send.command.command().clone();

                // Actually mark layers as eq
                res.update_equal_layers();

                // By returning the origin_command, it is basically the same as sending that layer
                res.highest_sent_layer = MarkerLayerId::AtLayer(first_layer_id);

                Ok(TreeCreationSuccess {
                    tree: res,
                    origin_command: Some(first_cmd),
                })
            }
        }
    }

    // returns the number of nodes this action creates
    #[instrument(
        name = "expand_fully",
        fields(
            description = "Expand all possible tree nodes (until hitting a limit or an unknown)"
        ),
        skip(self)
    )]
    fn expand_fully(&mut self) -> Result<TreeExpansionSuccess, TreeExpansionError> {
        info!(target: "strat", "EXPAND FULLY");
        info!(target: "strat", "At start: \n{}", self.tree_structure_str());
        info!(target: "strat", "highest_full_layer = {:?}", self.highest_full_layer);
        info!(target: "strat", "max_nodes = {:?}; node_count = {:?}", self.config.max_nodes, self.node_count);

        const MIN_LAYER: usize = 2;
        let node_budget = self.config.max_nodes.saturating_sub(self.node_count);
        let layer_budget = self
            .config
            .desired_depth
            .saturating_sub(self.full_layer_count());
        info!(target: "strat", "full_layer_count = {:?}", self.full_layer_count());

        info!(target: "strat", "node_budget = {node_budget}, layer_budget = {layer_budget}");

        let mut nodes_created = 0;
        let mut layers_fully_expanded = 0;

        let starting_full_layer = self.highest_full_layer;

        // Once a layer in-between was not fully expanded, that layer and all layers after that
        // will not incr the fully_expanded_layer-counter
        let mut prev_layer_not_fully_expanded = false;
        // Iterating through all the layers that are still to be expanded
        'expansion: for i in 0..layer_budget {
            // println!("Layerexpansion {i}/{layer_budget}");
            info!(target: "strat", "Layerexpansion {i}/{layer_budget}");
            // INFO: Can only break via node-budget if there are enough layers; otherwise, it will
            // keep trying to create new layers
            if (node_budget <= nodes_created && self.full_layer_count() >= MIN_LAYER)
                || layer_budget <= layers_fully_expanded
            {
                info!(target: "strat", "Out of budget");
                // Out of budget
                break;
            }

            // in step 0, it will be the lowest non-expanded layer
            let layer_to_expand = match starting_full_layer {
                MarkerLayerId::NotExistant => self.first_layer_absolute_id + RelativeLayerId(i),
                MarkerLayerId::AtLayer(l) => l + RelativeLayerId(i),
            };
            info!(target: "strat", "Expanding layer {layer_to_expand:?}");
            let _s = span!(
                Level::INFO,
                "expand_layer",
                link_layer_id = layer_to_expand.link()
            );
            let non_expanded_node_ids = {
                // println!("Layer to expand: {layer_to_expand:?}");
                let Some(layer) = self.layer_mut(layer_to_expand) else {
                    break 'expansion;
                };

                layer
                    .nodes
                    .iter()
                    .filter(|(_k, v)| v.applied_strategy.is_none())
                    .map(|(k, _v)| *k)
                    .collect::<Vec<_>>()
            };

            let num_of_expandables = non_expanded_node_ids.len();

            let mut failed_expansion_in_layer = false;
            let mut expanded = false;
            'layer_expansion: for non_expanded_node_id in non_expanded_node_ids {
                let abs_node_id = AbsoluteNodeId {
                    layer_id: layer_to_expand,
                    node_id: non_expanded_node_id,
                };
                let _s = span!(
                    Level::INFO,
                    "expand_node",
                    link_node_id = abs_node_id.link()
                );
                info!(target: "strat", link_node_id = abs_node_id.link(), "Expanding: {non_expanded_node_id:?}");

                let node_expansion_result = self.try_expand_node(abs_node_id);
                info!(target: "strat", "    Res = {node_expansion_result:?}");
                match node_expansion_result {
                    NodeExpansionResult::NotExpandable => {
                        error!(target: "strat", "Cannot expand node");
                        return Err(TreeExpansionError {
                            node: abs_node_id,
                            expansion: node_expansion_result,
                        });
                    }
                    NodeExpansionResult::NotYetExpandable => {
                        info!(target: "strat", "Could not expand yet --> skipped_layer = true");
                        prev_layer_not_fully_expanded = true;
                        // INFO: This should be alright, as long as the root node isn't being
                        // completed and is the only one left
                    }
                    NodeExpansionResult::AlreadyExpanded => {
                        error!(target: "strat", "Already expanded, but not marked as such");
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
                        expanded = true;
                        nodes_created += num_of_nodes;
                    }
                }

                if node_budget <= nodes_created && self.full_layer_count() >= MIN_LAYER {
                    info!(target: "strat", "Reached node budged");
                    break 'expansion;
                }
            }
            if prev_layer_not_fully_expanded {
                // We skipped some node expansion in this layer as it was not yet available to us
                // INFO: it will still try to expand the already existing layers up to that depth,
                // but it will not incr the expanden-layer-counter
                info!(target: "strat", "Cannot increase full layer; skipped previously");
            } else {
                layers_fully_expanded += 1;
                // The highest full layer is the layer after one in which all nodes were expanded
                if expanded || num_of_expandables == 0 {
                    // --> Either we expanded all nodes that we had to expand successfully, or
                    // there were no nodes to expand (they were headless)

                    if num_of_expandables == 0 {
                        info!(target: "strat", "Increasing full layer --> Layer is completely headless and thus fully expanded");
                    }

                    self.highest_full_layer =
                        MarkerLayerId::AtLayer(layer_to_expand + RelativeLayerId(1));
                } else if !expanded {
                    info!(target: "strat", "Did not increase full layer --> Did not do any expansions even though elements were expandable");
                }
                info!(target: "strat", "Increase full layers to {:?}", self.highest_full_layer);

                // Could be none since layer-expansion could fail completely
                if let Some(child_layer) = self.layer_mut(layer_to_expand + RelativeLayerId(1)) {
                    child_layer.is_full = true;
                }
            }
        }
        info!(target: "strat", "At end: \n{}", self.tree_structure_str());
        Ok(TreeExpansionSuccess {
            nodes: nodes_created,
            layers: layers_fully_expanded,
        })
    }

    fn full_layer_count(&self) -> usize {
        match self.highest_full_layer {
            MarkerLayerId::NotExistant => 0,
            MarkerLayerId::AtLayer(l) => {
                (l - self.first_layer_absolute_id)
                    .expect("Highest full layer should not be smaller than first layer")
                    .0
                    + 1
            }
        }
    }

    // Returns the number of children this operation created
    #[instrument(
        name = "try_expand_node",
        fields(
            description = "try expanding a single node",
            link_node_id = node_id.link()
        ),
        skip(self)
    )]
    fn try_expand_node(&mut self, node_id: AbsoluteNodeId) -> NodeExpansionResult {
        let mut nodes_created = 0;
        let goal_position = self.goal_position;

        // INFO: ############# Getting the node ##############################
        let node = self
            .node_mut(node_id)
            .expect("Passed inside the tree; should be valid");
        // INFO: ############# Checking that it isn't already expanded ##############################
        if let Some(applied_strategy) = node.applied_strategy.as_ref() {
            match applied_strategy {
                Err(strategy_end) => {
                    info!(target: "strat", "Precomputed EndState {strategy_end:?}");
                    return NodeExpansionResult::EndState(strategy_end.clone());
                }
                _ => {
                    info!(target: "strat", "Already Expanded");
                    return NodeExpansionResult::AlreadyExpanded;
                }
            }
        }
        // if node.applied_strategy.is_some() {
        //     // The node already is fully expanded
        //     info!(target: "strat", "Alreay Expanded");
        //     return NodeExpansionResult::AlreadyExpanded;
        // }

        let basis_world = &node.on_basis_of_world;
        info!(target: "strat", "BASED ON WORLD: \n{basis_world}");

        // INFO: ############# Checking that it has a strategy needed for expansion ##############################
        let Some(basis_strategy_state) = &node.on_basis_of_state else {
            info!(target: "strat", "NO ASSOCIATED STRATEGY --> Not expandable");
            // There is no strategy to base this expansion on
            return NodeExpansionResult::NotExpandable;
        };

        // INFO: ############# If the strategy does not return a proper value, it is just not yet ready ##############################
        let StrategyComputationResult::Computed(expansion_actions) =
            basis_strategy_state.next_cmd(basis_world, &goal_position)
        else {
            info!(target: "strat", "Not yet expandable");
            return NodeExpansionResult::NotYetExpandable;
        };
        info!(target: "strat", "EXPANSION: \n{expansion_actions:#?}");

        let expansion_actions: ComputedActions<N, S> = match expansion_actions {
            Ok(o) => o,
            Err(e) => {
                info!(target: "strat", "=> Expanded to EndState");
                // Only trigger this error if this node is reached
                // The node expansion has shown that the node we expanded is not expandable
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
            info!(target: "strat", "Applying action: {expansion_action:?}");
            let do_cmd = expansion_action.after_command;
            let strategy_state_after = expansion_action.next_strategy_state;

            // At the first step there should only be one node to expand, but further down the line
            // there could be an entire collection of them
            let mut new_apply_on_node = vec![];
            for parent_node in apply_on_node {
                info!(target: "strat",link_node_id = parent_node.link(), "Based on node {parent_node:?}");
                let child_node_layer = parent_node.layer_id + RelativeLayerId(1);
                let (parent_pos, cmd_application) = {
                    let basis_world = {
                        let parent_node = self.node(parent_node).expect("Checked");
                        &parent_node.on_basis_of_world
                    };

                    (
                        basis_world.mouse,
                        FilteredCommandApplication::new(
                            Some(basis_world.clone().into()),
                            do_cmd.clone(),
                        ),
                    )
                };

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

                    self.visuals.create_path(
                        path_id.clone(),
                        PathSegment::new(parent_pos, child_world.mouse)
                            .expect("Child should form valid path"),
                    );
                    // TODO: maybe reset child_world
                    let child_node = StrategyTreeNode::new_leaf(
                        child_world.clone(),
                        strategy_state_after.clone(),
                        path_id,
                    );
                    let child_node_id = { self.add_node(child_node, child_node_layer) };
                    info!(target: "strat", link_node_id = child_node_id.link(), "CHILD NODE {child_node_id:?}\n{child_world}");
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

    #[instrument(
        name = "add_node",
        fields(
            description = "Add new node to layer",
            link_layer_id = to_layer.link()
        ),
        skip(self, node)
    )]
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

    #[instrument(
        name = "fill_layers_to_id",
        fields(
            description = "Create new layers until reaching that id",
            link_layer_id = to_layer.link()
        ),
        skip(self)
    )]
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

    // fn fully_expanded_layer_count(&self) -> usize {
    //     self.highest_full_layer + 1
    // }

    // fn highest_full_layer(&self) -> AbsoluteLayerId {
    //     self.first_layer_absolute_id + RelativeLayerId(self.highest_full_layer)
    // }

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
            Some((layer_id - self.first_layer_absolute_id)?)
        } else {
            None
        }
    }

    #[instrument(
        name = "handle_finish_root",
        fields(
            description = "Treat the current root as finished, removing it from the tree and replacing it with its 1 (!!!) successor; THEN: Expand",
        ),
        skip(self)
    )]
    pub fn handle_finish_root(&mut self) -> Result<Vec<Command>, StrategyTreeError> {
        let root_id = self.root_node();
        let root_node = self.node(root_id).expect("Root has to exist");
        let root_cmd = root_node
            .applied_strategy
            .as_ref()
            .expect("Checked")
            .as_ref()
            .expect("Cannot be an end-state")
            .command
            .command();
        info!(target: "strat", "FINISHING ROOT {root_id:?} with command {root_cmd:?}: \n{}", root_node.on_basis_of_world);
        let commands = self.finish_root();

        match commands.as_ref() {
            Err(FinishRootError::MultipleSuccessors(successors)) => {
                error!(target: "strat", "SUCCESSOR LIST:");
                for (path_id, node_id) in successors {
                    let node = self.node(*node_id).expect("Node should exist");
                    let map = &node.on_basis_of_world;
                    error!(target: "strat", "SUCCESSOR {path_id:?}\n{map}");
                }
                panic!("Multiple successors");
            }
            Ok(cmds) => {
                let root_id = self.root_node();
                let root_node = self.node(root_id).expect("Root has to exist");
                info!(target: "strat", "NEW ROOT {root_id:?}: \n{}", root_node.on_basis_of_world);
            }
            o => info!(target: "strat", "RES: {o:?}"),
        }
        self.expand_fully()?;
        commands.map_err(StrategyTreeError::from)
    }

    // removes the root, making its successor the new root; if the successor was not yet expanded
    // into a command, it will do so at this point, returning its NodeAction
    #[instrument(
        name = "finish_root",
        fields(
            description = "Treat the current root as finished, removing it from the tree and replacing it with its 1 (!!!) successor",
        ),
        skip(self)
    )]
    pub fn finish_root(&mut self) -> Result<Vec<Command>, FinishRootError> {
        // println!("Finish root");
        let (_outcome_id, successor) = {
            let root_id = self.root_node();
            info!(target: "strat", link_node_id = root_id.link(), "FINISHING NODE {root_id:?}");
            // WARN: Will not expand the root before getting its successor; Any successor should
            // already have been determined by the preceding measurements
            let root = self.node_mut(root_id).expect("Checked");

            let Some(strategy_res) = &root.applied_strategy else {
                // The root will be expanded, at least when it is placed into the root-slot (since the
                // execution starts there and the potential outcomes have to be known)
                // WARN: Root can only be sent if it is expanded (send criterion: is_eq + next
                // layer fully expanded)
                error!(target: "strat", link_node_id = root_id.link(), "ROOT cannot have been executed as it has not been expanded");
                return Err(FinishRootError::RootNotExpanded);
            };

            let node_action = match strategy_res {
                Ok(n) => n,
                Err(e) => {
                    // Per definition, the command that just finished cannot be a command which was
                    // not executable
                    error!(target: "strat", link_node_id = root_id.link(), "ROOT cannot have been executed as it does not have an associated command");
                    return Err(FinishRootError::ImpossibleRootAction(e.clone()));
                }
            };

            let potential_outcomes = &node_action.potential_outcomes;
            let next_action_len = potential_outcomes.len();

            info!(target: "strat", link_node_id = root_id.link(), "Potential successors ({next_action_len}): \n{potential_outcomes:#?}");

            match next_action_len {
                0 => {
                    error!(target: "strat", link_node_id = root_id.link(), "ROOT does not have a successor");
                    return Err(FinishRootError::NoSuccessor);
                }
                1 => {
                    let (o, c) = potential_outcomes.iter().next().unwrap();
                    info!(target: "strat", link_node_id = c.link(), "SUCCESSOR = {c:?}");
                    (*o, *c)
                }
                2.. => {
                    error!(target: "strat", link_node_id = root_id.link(), "ROOT has multiple successors");
                    // TODO: print out all the possibilities
                    return Err(FinishRootError::MultipleSuccessors(
                        potential_outcomes.clone(),
                    ));
                }
            }
        };

        let next_root_expansion_res = self.try_expand_node(successor);
        info!(target: "strat", link_node_id = successor.link(), "SUCCESSOR Expansion ~~> {next_root_expansion_res:?}");
        match next_root_expansion_res {
            NodeExpansionResult::AlreadyExpanded => {
                info!(target: "strat", link_node_id = successor.link(), "Already expanded; No further processing");
                // Best case
            }
            NodeExpansionResult::NotExpandable => {
                error!(target: "strat", link_node_id = successor.link(), "Successor can never be expanded; Does not have a strategy");
                return Err(FinishRootError::SuccessorNotExpandable);
            }
            NodeExpansionResult::NotYetExpandable => {
                error!(target: "strat", link_node_id = successor.link(), "Successor can never be expanded; Waiting for more measurements, but there are no more coming");
                return Err(FinishRootError::SuccessorNotYetExpandable);
            }
            NodeExpansionResult::EndState(s) => {
                info!(target: "strat", link_node_id = successor.link(), "Successor marks end of strategy_execution; Will need new strategy");
                self.layers.remove(0);
                self.node_count -= 1;

                let new_first_layer = self.layers.first_mut().expect("Has Successor");
                self.first_layer_absolute_id = new_first_layer.absolute_layer_id;

                return Err(FinishRootError::SuccessorIsEnd(s));
            }
            NodeExpansionResult::Expanded(_) => {
                info!(target: "strat", link_node_id = successor.link(), "Successor successfully expanded");
                // We know that this is expanding a full layer, since the new root layer only
                // contains the successor
                // layer after the successor (which was just expanded) is now full
                self.highest_full_layer = self.highest_full_layer.max(MarkerLayerId::AtLayer(
                    *successor.layer_id() + RelativeLayerId(1),
                ));
            }
        }

        self.layers.remove(0);
        self.node_count -= 1;

        let new_first_layer = self.layers.first_mut().expect("Has Successor");
        // new_first_layer.is_full = true;
        self.first_layer_absolute_id = new_first_layer.absolute_layer_id;

        self.layer_mut(self.first_layer_absolute_id + RelativeLayerId(1))
            .expect("Just created (and filled) it")
            .is_full = true;

        info!(target: "strat", "New first_layer = {:?}", self.first_layer_absolute_id);

        self.update_equal_layers();
        Ok(self.new_sends())
    }

    #[instrument(
        name = "prune_not_potentially_eq",
        fields(
            description = "Prunes all command-outcomes of all paths which do not match the given filter; ALSO: Apply the full filter if it could lead a node to becoming expandable",
        ),
        skip(self)
    )]
    fn tree_structure_str(&self) -> String {
        let mut nodes = VecDeque::new();
        let mut res = "".to_string();
        nodes.push_front(self.root_node());
        while let Some(node_id) = nodes.pop_front() {
            let children = self.node(node_id).expect("Should exist").children();
            if let Some(children) = children {
                for child in children.values() {
                    nodes.push_front(*child);
                }
            }

            let depth = node_id.layer_id - self.first_layer_absolute_id;
            let indent = " ".repeat(depth.map(|d| d.0).unwrap_or(0) * 4);
            res = format!("{res}\n{indent}> {node_id:?}");
        }
        res
    }
    pub fn prune_not_potentially_eq(&mut self, filter: &Map<N>) -> Result<(), PruneError> {
        // println!("Prune");
        for layer_offset in 1..self.layers.len() {
            let layer_id = self.first_layer_absolute_id + RelativeLayerId(layer_offset);
            let Some(layer) = self.layer_mut(layer_id) else {
                return Err(PruneError::UnknownNode(AbsoluteNodeId {
                    layer_id,
                    node_id: RelativeNodeId(0),
                }));
            };
            let prune_nodes = layer
                .nodes
                .iter_mut()
                .filter_map(|(node_id, node)| {
                    if node.on_basis_of_world.map.potentially_eq(filter) {
                        if node.on_basis_of_state.is_some() && node.applied_strategy.is_none() {
                            // It is worth expanding the map (the node is expandable, but not expanded)
                            node.on_basis_of_world.map = node
                                .on_basis_of_world
                                .map
                                .union(filter)
                                .expect("Is potentially_eq");
                        }
                        None
                    } else {
                        Some(*node_id)
                    }
                })
                .collect::<Vec<_>>();
            info!(target: "strat", "Pruning layer = {layer_id:?}, nodes = {prune_nodes:?}");

            for node_id in prune_nodes {
                let abs_node_id = AbsoluteNodeId { layer_id, node_id };
                self.prune_node(abs_node_id)?;
            }
        }

        Ok(())
    }

    #[instrument(
        name = "prune_current_command_by_rejection",
        fields(
            description = "Prunes branches of the current command using rejection-events from the micromouse manager",
        ),
        skip(self)
    )]
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

    #[instrument(skip(self), name = "prune_node", fields(link_node_id = node_and_children.link(), description = "Delete a node and all of its children"))]
    fn prune_node(&mut self, node_and_children: AbsoluteNodeId) -> Result<(), PruneError> {
        let Some(node_to_delete) = self.node(node_and_children) else {
            return Err(PruneError::UnknownNode(node_and_children));
        };

        if let Some((child_id, parent)) = node_to_delete.as_branch_from_parent.clone().map(|p| {
            self.visuals.remove_path(p.clone());
            (
                p.branch,
                self.node_mut(p.from_node).expect("Parent should exist"),
            )
        }) {
            parent
                .children_mut()
                .expect("Parent should have children")
                .remove(&child_id);
        }

        let mut to_delete = vec![node_and_children];
        while let Some(delete_node) = to_delete.pop() {
            let Some(n) = self.node(delete_node) else {
                return Err(PruneError::UnknownNode(delete_node));
            };
            let mut children = unsafe { self.delete_node_no_clean(delete_node) }?;
            to_delete.append(&mut children);
        }

        Ok(())
    }

    #[instrument(
        name = "prune_branch",
        fields(
            description = "Remove a specific branch and all of its descendants",
            link_branch_id = branch.link(),
        ),
        skip(self)
    )]
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

        self.prune_node(first_delete_node)
    }

    // Returns the nodes children
    #[instrument(
        name = "delete_node_no_clean",
        fields(
            description = "Deletes the node, but does not do any cleanup",
            link_node_id = node.link(),
        ),
        skip(self)
    )]
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

    #[instrument(
        name = "update_equal_layers",
        fields(description = "Update the highest eq layer id (in case there are new eq-layers)",),
        skip(self)
    )]
    fn update_equal_layers(&mut self) {
        let highest_full_layer = self.highest_full_layer;
        let highest_eq_layer = self.highest_eq_layer;

        let highest_full_layer_offset = match self.highest_full_layer {
            MarkerLayerId::NotExistant => {
                error!(target: "strat", "There is not a single full layer while updating eq layers; Should only be the case at startup");
                -1
            }
            MarkerLayerId::AtLayer(l) if l + RelativeLayerId(1) == self.first_layer_absolute_id => {
                info!(target: "strat", "highest full layer is directly below the current first_layer");
                -1
            }
            MarkerLayerId::AtLayer(l) => {
                info!(target: "strat", "highest_full_layer = {highest_full_layer:?}");
                (l - self.first_layer_absolute_id)
                    .expect("highest_full_layer should be >= first layer")
                    .0 as i32
            }
        };
        let highest_eq_layer_offset = match self.highest_eq_layer {
            MarkerLayerId::NotExistant => {
                // error!(target: "strat", "There is not a single eq layer while updating eq layers; Should only be the case at startup");
                -1
            }
            MarkerLayerId::AtLayer(l) if l + RelativeLayerId(1) == self.first_layer_absolute_id => {
                info!(target: "strat", "highest eq layer is directly below the current first_layer");
                -1
            }
            MarkerLayerId::AtLayer(l) => {
                info!(target: "strat", "highest_eq_layer = {highest_eq_layer:?}");
                (l - self.first_layer_absolute_id)
                    .expect("highest_eq_layer should be >= first layer")
                    .0 as i32
            }
        };

        info!(target: "strat", "Rechecking layer[{}..={}]", highest_eq_layer_offset + 1, highest_full_layer_offset);

        for layer_offset in highest_eq_layer_offset + 1..=highest_full_layer_offset {
            let layer_id = self.first_layer_absolute_id + RelativeLayerId(layer_offset as usize);
            if !self
                .layer_mut(layer_id)
                .expect("Should always be in bounds")
                .update_eq_command()
            {
                info!(target: "strat", link_layer_id = layer_id.link(),"{layer_id:?} is not EQ");
                break;
            }
            info!(target: "strat", link_layer_id = layer_id.link(),"{layer_id:?} is EQ");
            self.highest_eq_layer = MarkerLayerId::AtLayer(layer_id);
        }
    }

    #[instrument(
        name = "new_sends",
        fields(
            description = "Moves the send-cursor further (if possible) and returns an ordered list of newly sent commandy",
        ),
        skip(self)
    )]
    fn new_sends(&mut self) -> Vec<Command> {
        info!(target: "strat", "highest_full_layer = {:?}", self.highest_full_layer);
        info!(target: "strat", "highest_eq_layer = {:?}", self.highest_eq_layer);
        info!(target: "strat", "highest_sent_layer = {:?}", self.highest_sent_layer);

        let highest_full_layer_offset = match self.highest_full_layer {
            MarkerLayerId::NotExistant => {
                error!(target: "strat", "Highest full layer does not exist, but should");
                return vec![];
            }
            MarkerLayerId::AtLayer(l) => l.0 as i32 - self.first_layer_absolute_id.0 as i32,
        };

        // todo!("Need to make the exact condition more clear");

        if self.highest_eq_layer <= self.highest_sent_layer {
            info!(target: "strat", "Only eq layer is smaller than sent layer");
            return vec![];
        }

        let highest_eq_layer_offset = match self.highest_eq_layer {
            MarkerLayerId::NotExistant => {
                error!(target: "strat", "Highest eq layer does not exist, but should");
                return vec![];
            }
            MarkerLayerId::AtLayer(l) => l.0 as i32 - self.first_layer_absolute_id.0 as i32,
        };

        let highest_sent_layer_offset = match self.highest_sent_layer {
            MarkerLayerId::NotExistant => {
                error!(target: "strat", "Highest sent layer does not exist, but should");
                return vec![];
            }
            // MarkerLayerId::AtLayer(l) if l == self.first_layer_absolute_id => return vec![],
            MarkerLayerId::AtLayer(l) => l.0 as i32 - self.first_layer_absolute_id.0 as i32,
        };

        // INFO:
        // The next layer needs to be full:
        // This is because the tree may close down, in which case the grafting points are the
        // branches going out from the last sent layer; after removing the strategy (or in
        // generally), we have no way of expanding them just in time, which means, that we must
        // constrain the send here

        let highest_sendable_layer_offset =
            i32::min(highest_eq_layer_offset, highest_full_layer_offset - 1);
        // let highest_sendable_layer = MarkerLayerId::AtLayer(
        //     self.first_layer_absolute_id
        //         + RelativeLayerId(
        //             highest_eq_layer_offset
        //                 .min(highest_full_layer_offset.0 - 1),
        //         ),
        // );

        info!(target: "strat", "highest_full_layer_offset = {highest_full_layer_offset:?}");
        info!(target: "strat", "highest_eq_layer_offset = {highest_eq_layer_offset:?}");
        info!(target: "strat", "highest_sent_layer_offset = {highest_sent_layer_offset:?}");
        info!(target: "strat", "highest_sendable_layer_offset = {highest_sendable_layer_offset:?}");

        // let highest_sendable_layer_offset = match highest_sendable_layer {
        //     MarkerLayerId::NotExistant => {
        //         // error!(target: "strat", "Highest sendable layer does not exist, but should");
        //         return vec![];
        //     }
        //     MarkerLayerId::AtLayer(l) if l == self.first_layer_absolute_id => return vec![],
        //     MarkerLayerId::AtLayer(l) => l - self.first_layer_absolute_id,
        // };

        info!(target: "strat", "Sending: [{}..={}]", highest_sent_layer_offset + 1, highest_sendable_layer_offset);

        if highest_sent_layer_offset < highest_sendable_layer_offset {
            let l = self.layers
                [(highest_sent_layer_offset + 1) as usize..=highest_sendable_layer_offset as usize]
                .iter()
                .map(|l| l.eq.as_ref().expect("Explicitly should exist").clone())
                .collect();

            self.highest_sent_layer = MarkerLayerId::AtLayer(
                self.first_layer_absolute_id
                    + RelativeLayerId(highest_sendable_layer_offset as usize),
            );
            return l;
        }
        vec![]
    }

    #[instrument(name = "merge", fields(description = "MERGE",), skip(self))]
    fn merge(&mut self) // -> ?
    {
        // Try merging non-eq-layers
        todo!("Nice-to-have, but really complex")
    }

    #[instrument(
        name = "transform_command",
        fields(
            description = "Used for merge; Tries to split a command (and its node) into 2",
            link_node_id = from_node.link(),
        ),
        skip(self)
    )]
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
    #[instrument(
        name = "move_node_back",
        fields(
            description = "Moves a node back a layer (which is unsafe as it creates a link jumping a layer)",
            link_node_id = node_id.link()
        ),
        skip(self)
    )]
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
    #[instrument(
        name = "handle_map_update",
        fields(description = "Prune, expand, send",),
        skip(self)
    )]
    pub fn handle_map_update(&mut self, map: &Map<N>) -> Result<Vec<Command>, StrategyTreeError> {
        info!(target: "strat", "HANDLE CMD MAP UPDATE\n{map}");
        if self
            .node(self.root_node())
            .expect("Index should be valid")
            .on_basis_of_world
            .map
            .potentially_eq(&map)
        {
            self.prune_not_potentially_eq(map)?;
            let _ = self.expand_fully()?;
            // TODO: self.merge()?;

            self.update_equal_layers();
            Ok(self.new_sends())
        } else {
            Err(StrategyTreeError::MapConflictsWithMapUpdate)
        }
    }

    // Filters the tree using the rejection-events from the currently processed command
    #[instrument(
        name = "handle_cmd_rejection",
        fields(description = "Prune, expand, send",),
        skip(self)
    )]
    pub fn handle_cmd_rejection(
        &mut self,
        rejections: &RejectedOutcomes,
    ) -> Result<Vec<Command>, StrategyTreeError> {
        info!(target: "strat", "HANDLE CMD REJECTION");
        self.prune_current_command_by_rejection(rejections)?;
        let _ = self.expand_fully()?;
        self.update_equal_layers();
        Ok(self.new_sends())
    }

    #[instrument(
        name = "close",
        fields(
            description = "Close the current strategy_tree, leaving behind the commands that were already sent and thus cannot be taken back",
        ),
        skip(self)
    )]
    pub fn close(self) -> (FrontendVisuals, SentUnfinishedCommands<N>) {
        info!(target: "strat", "CLOSE");
        let highest_sent_layer = self.highest_sent_layer;

        //INFO: We also need to include the last layer which was not yet sent, but was expanded
        //from the last sent layer; it is our grafting-point
        let highest_sent_layer_and_exp = match highest_sent_layer {
            MarkerLayerId::NotExistant => 0,
            MarkerLayerId::AtLayer(l) if l < self.first_layer_absolute_id => {
                let root_world = self
                    .node(self.root_node())
                    .as_ref()
                    .expect("Root must exist")
                    .on_basis_of_world
                    .deref()
                    .clone();
                return (
                    self.visuals,
                    SentUnfinishedCommands::HasBlockingRoot { world: root_world },
                );
            }
            MarkerLayerId::AtLayer(l) => (l - self.first_layer_absolute_id).expect("Checked").0 + 1,
        };

        if highest_sent_layer <= MarkerLayerId::AtLayer(self.first_layer_absolute_id)
            && self
                .node(self.root_node())
                .expect("HAS TO EXIST")
                .applied_strategy
                .as_ref()
                .is_some_and(|a| a.is_err())
        {
            //INFO: The current root is blocking; it appears to be sent, but  it is not / cannot be
            //(A blocking root is the only root that is not sendable)

            let world = self
                .node(self.root_node())
                .expect("Checked")
                .on_basis_of_world
                .clone()
                .into();

            return (
                self.visuals,
                SentUnfinishedCommands::HasBlockingRoot { world },
            );
        }

        let mut layers = vec![];
        let mut inner_layers = self.layers;
        for _i in 0..=highest_sent_layer_and_exp {
            // Remove the lowest layer
            let layer = inner_layers.remove(0);

            layers.push(layer.try_into().expect(
                "Within highest sent layer, should fulfil all the properties of the sent layer",
            ))
        }

        (
            self.visuals,
            SentUnfinishedCommands::HasQueue {
                layers: layers.non_empty().expect("Root layer has to have been sent; meaning that it and its children can form highest_sent + exp"),
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Error, Deserialize)]
pub enum PruneError {
    #[error("Encountered unknown node: {0:?}")]
    UnknownNode(AbsoluteNodeId),
    #[error("Root is not prunable (root = {0:?})")]
    CannotDeleteRoot(AbsoluteNodeId),
    #[error("No child to be pruned (source = {0:?})")]
    SourceHasNoChildren(AbsoluteNodeId),
    #[error("Tried to prune child that does not exist ({0:?})")]
    SourceDoesNotHaveThisChild(AbsolutePathId),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AbsoluteNodeId {
    layer_id: AbsoluteLayerId,
    node_id: RelativeNodeId,
}

impl AbsoluteNodeId {
    pub fn layer_id(&self) -> &AbsoluteLayerId {
        &self.layer_id
    }
    pub fn node_id(&self) -> &RelativeNodeId {
        &self.node_id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbsolutePathId {
    pub from_node: AbsoluteNodeId,
    pub branch: PathLocalOutcomeId,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct RelativeNodeId(pub usize);

#[derive(Debug)]
pub enum SentUnfinishedCommands<const N: usize> {
    HasQueue {
        // They are Strategy-Agnostic by not allowing the nodes to expand, which allows us to leave out
        // the strategy-state, which is needed for expansion
        layers: NonEmpty<Vec<SentTreeLayer<N>>>,
    },
    // WARN: If there is no queue here, it would mean that the tree is not entirely empty (it
    // cannot be), but there is a root which is blocking (i.e. it marks a StrategyEndState)
    //
    // This means, that we can still pull the "current_state" from this root, although it will not
    // lead to any expansion on its own (and cannot be sent)
    HasBlockingRoot {
        world: WorldData<N>,
    },
}

impl Sub for AbsoluteLayerId {
    type Output = Option<RelativeLayerId>;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.0 < rhs.0 {
            None
        } else {
            Some(RelativeLayerId(self.0 - rhs.0))
        }
    }
}

impl Sub<RelativeLayerId> for AbsoluteLayerId {
    type Output = AbsoluteLayerId;

    fn sub(self, rhs: RelativeLayerId) -> Self::Output {
        AbsoluteLayerId(self.0 - rhs.0)
    }
}

impl Add<RelativeLayerId> for AbsoluteLayerId {
    type Output = AbsoluteLayerId;
    fn add(self, rhs: RelativeLayerId) -> Self::Output {
        AbsoluteLayerId(self.0 + rhs.0)
    }
}

impl<const N: usize, S: Strategy<N>> StrategyTreeLayer<N, S> {
    #[instrument(name = "non_expandable_from_cleaned", fields(link_layer_id = sent_layer.absolute_layer_id.link(), description = "Turns an erased layer into a typed strategy tree layer that has no type-specific value"))]
    pub fn non_expandable_from_cleaned(sent_layer: SentTreeLayer<N>) -> Self {
        let nodes = sent_layer.nodes.into_iter();
        let transformed_nodes = nodes
            .map(|(id, val)| {
                (
                    id,
                    StrategyTreeNode::<N, S>::non_expandable_from_cleaned(val),
                )
            })
            .collect::<HashMap<_, _>>();
        StrategyTreeLayer {
            nodes: transformed_nodes,
            absolute_layer_id: sent_layer.absolute_layer_id,
            is_full: true,
            eq: Some(sent_layer.eq),
            node_count: sent_layer.node_count,
            // This is not technically correct, but it should not be used after this
            // point, so creating a nonsense value is ok
            node_id_counter: RelativeNodeIdCounter(0),
            is_sent: true,
        }
    }
    pub fn node(&self, node_id: RelativeNodeId) -> Option<&StrategyTreeNode<N, S>> {
        self.nodes.get(&node_id)
    }
    pub fn node_mut(&mut self, node_id: RelativeNodeId) -> Option<&mut StrategyTreeNode<N, S>> {
        self.nodes.get_mut(&node_id)
    }
    #[instrument(name = "new StrategyTreeLayer", fields(link_layer_id = absolute_layer_id.link()))]
    pub fn new(absolute_layer_id: AbsoluteLayerId) -> Self {
        Self {
            node_id_counter: RelativeNodeIdCounter::default(),
            nodes: HashMap::new(),
            absolute_layer_id,
            is_full: false,
            eq: None,
            node_count: 0,
            is_sent: false,
        }
    }

    #[instrument(name = "add_node", skip(self, node), fields(link_layer_id = self.absolute_layer_id.link()))]
    pub fn add_node(&mut self, node: StrategyTreeNode<N, S>) -> RelativeNodeId {
        let next_id = self.node_id_counter.next();
        self.nodes.insert(next_id, node);
        self.node_count += 1;
        next_id
    }

    #[instrument(name = "delete_node", skip(self), fields(link_layer_id = self.absolute_layer_id.link(), link_node_id = AbsoluteNodeId{ layer_id: self.absolute_layer_id, node_id: node }.link()))]
    pub unsafe fn delete_node(
        &mut self,
        node: RelativeNodeId,
    ) -> Result<Vec<AbsoluteNodeId>, PruneError> {
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

    #[instrument(
        name = "equal_command",
        skip(self),
        fields(
            description = "Check whether a layer's commands are all equal and return it if it exists"
        )
    )]
    pub fn equal_command(&self) -> Option<Command> {
        if !self.is_full {
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
    #[instrument(
        name = "update_eq_command",
        skip(self),
        fields(description = "Update the layer's common command")
    )]
    pub fn update_eq_command(&mut self) -> bool {
        if self.eq.is_some() {
            return true;
        }
        if !self.is_full {
            return false;
        }
        self.eq = self.equal_command();
        if self.eq.is_some() {
            return true;
        }
        false
    }
}

#[derive(Default)]
pub struct RelativeNodeIdCounter(pub usize);

#[derive(Debug, Clone, Serialize, Error, Deserialize)]
pub enum FinishRootError {
    #[error("The root has no successor; it could not be expanded")]
    NoSuccessor,
    #[error("The root has multiple successors; Only 1 should exist at this point")]
    MultipleSuccessors(HashMap<PathLocalOutcomeId, AbsoluteNodeId>),
    #[error("The root could not be expanded, but it should have 1 successor")]
    RootNotExpanded,
    #[error("The root was declared finished, but it was not a node that was executable")]
    ImpossibleRootAction(StrategyEndState),
    #[error("The root cannot have children and thus cannot have a successor")]
    SuccessorNotExpandable,
    #[error("The root cannot yet have children due to not having enough information, but due to finishing the command, there is no more information available")]
    SuccessorNotYetExpandable,
    // Either a valid end or a strategy error
    #[error("The successor marks the end of this strategy's execution; Not really an error, just an info")]
    SuccessorIsEnd(StrategyEndState),
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
    #[instrument(name = "new_leaf", skip(state_at_start_of_node), fields(description = "Create new leaf-node at end of path", link_branch_id = at.link()))]
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

    #[instrument(
        name = "new_orphan",
        skip(state_at_start_of_node),
        fields(description = "Create new node without parent")
    )]
    pub fn new_orphan(
        world_at_start_of_node: PartialWorldData<N>,
        state_at_start_of_node: Option<S>,
    ) -> Self {
        Self {
            on_basis_of_world: world_at_start_of_node,
            on_basis_of_state: state_at_start_of_node,
            applied_strategy: None,
            as_branch_from_parent: None,
        }
    }

    #[instrument(
        name = "non_expandable_from_cleaned",
        fields(
            description = "Converts an erased node back to a typed node (but without type-specific values)"
        )
    )]
    pub fn non_expandable_from_cleaned(cleaned: SentCommandNode<N>) -> Self {
        Self {
            on_basis_of_world: cleaned.on_basis_of_world,
            on_basis_of_state: None,
            applied_strategy: Some(cleaned.applied_strategy),
            as_branch_from_parent: cleaned.as_branch_from_parent,
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

#[derive(Debug, Copy, Clone)]
pub enum LayerReductionError {
    NodesNotExpanded,
    LayerNotFull,
    LayerNotEq,
    LayerNotSent,
}

impl<const N: usize, S: Strategy<N>> TryFrom<StrategyTreeLayer<N, S>> for SentTreeLayer<N> {
    type Error = LayerReductionError;
    #[instrument(name = "try_from StrategyTreeLayer", skip(value), fields(link_layer_id = value.absolute_layer_id.link(), description = "Erase layer's strategy"))]
    fn try_from(value: StrategyTreeLayer<N, S>) -> Result<Self, Self::Error> {
        if !value.is_full {
            error!(target: "strat", link_layer_id = value.absolute_layer_id.link(), "Layer that is erased is not full");
            return Err(LayerReductionError::LayerNotFull);
        }
        if value.eq.is_none() {
            error!(target: "strat", link_layer_id = value.absolute_layer_id.link(), "Layer that is erased is not equal");
            return Err(LayerReductionError::LayerNotEq);
        }
        if value.is_sent {
            error!(target: "strat", link_layer_id = value.absolute_layer_id.link(), "Layer that is erased is not sent");
            return Err(LayerReductionError::LayerNotSent);
        }

        let node_len = value.node_count;

        let nodes = value
            .nodes
            .into_iter()
            .filter_map(|(k, v)| match SentCommandNode::try_from(v) {
                Ok(n) => Some((k, n)),
                Err(e) => None,
            })
            .collect::<HashMap<_, _>>();

        if node_len != nodes.len() {
            return Err(LayerReductionError::NodesNotExpanded);
        }

        Ok(SentTreeLayer {
            eq: value.eq.expect("Checked"),
            nodes,
            absolute_layer_id: value.absolute_layer_id,
            node_count: node_len,
        })
    }
}

impl<const N: usize, S: Strategy<N>> TryFrom<StrategyTreeNode<N, S>> for SentCommandNode<N> {
    type Error = ();
    fn try_from(value: StrategyTreeNode<N, S>) -> Result<Self, Self::Error> {
        Ok(Self {
            on_basis_of_world: value.on_basis_of_world,
            applied_strategy: value.applied_strategy.ok_or(())?,
            as_branch_from_parent: value.as_branch_from_parent,
        })
    }
}
