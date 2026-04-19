use crate::{
    comm::{micromouse_manager::MicromouseManager, website::FrontendManager},
    strategy::{strategy::Strategy, strategy_tree::StrategyTree},
};

pub struct Process<const N: usize> {
    micromouse_manager: MicromouseManager<N>,
    frontend_manager: FrontendManager,
    // strategy_tree: DynamicStrategyTree<N>
}
