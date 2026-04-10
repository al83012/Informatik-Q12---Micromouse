use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    comm::micromouse_message::Command,
    map::{
        command_world_state::{
            CommandOutcomes, CommandTerminationReason, PathLocalInterruptId, PathLocalOutcomeId,
        },
        map::{Map, PartialMap},
        world_data::{PartialWorldData, WorldData},
    },
    strategy::strategy::Strategy,
};

pub struct StrategyTree<const N: usize, S: Strategy<N> + Clone> {
    config: StrategyTreeConfig<N, S>,
    command_currently_processed: CurrentlyProcessedTreeNode<N>,
    certain_commands: MergedCommandLayers,
}

/// The MergedCommandLayers represent the commands that are common between all the commands of a
/// layer
/// (This also includes the case, in which only 1 command is left (or 1 command value split across
/// multiple))
pub struct MergedCommandLayers(pub Vec<Option<Command>>);

pub enum StrategyTreeError {
    MultipleSuccessors,
    NoSuccessor,
}

pub struct StrategyTreeConfig<const N: usize, S: Strategy<N> + Clone> {
    pub strategy_config: S::Config,
    pub desired_depth: usize,
    pub max_nodes: usize,
}

// = Layer 0
// Doesn't need a command or basis as that is all stored within the Micromouse Manager
pub struct CurrentlyProcessedTreeNode<const N: usize> {
    pub potential_outcomes: HashMap<PathLocalOutcomeId, StrategyTreeNode<N>>,
}

// = Layer 1..
pub struct StrategyTreeNode<const N: usize> {
    pub on_basis_of: PartialWorldData<N>,
    pub do_command: Command,
    pub potential_outcomes: HashMap<PathLocalOutcomeId, StrategyTreeNode<N>>,
}

impl<const N: usize, S> StrategyTree<N, S>
where
    // For use in the Strategy-Tree, we need to be able to duplicate the StrategyState to branch
    S: Strategy<N> + Clone,
{
    pub fn new(start_world: WorldData<N>, strategy_config: S::Config) -> Self {
        todo!()
    }

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
}

impl<const N: usize> StrategyTreeNode<N> {
    fn expand_node(&mut self /*, ? */) // -> ?
    {
        todo!("Expand ")
    }
}
impl<const N: usize> CurrentlyProcessedTreeNode<N> {
    fn expand_node(&mut self /*, ? */) // -> ?
    {
        todo!("Expand ")
    }
}
