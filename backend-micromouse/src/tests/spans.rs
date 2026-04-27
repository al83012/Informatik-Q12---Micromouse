use tracing::info;

use crate::{
    comm::micromouse_message::CommandId,
    strategy::strategy_tree::AbsoluteNodeId,
    utils::hyperlink_logging::{init_tree_logger, process_span, LinkFileName},
};

#[test]
#[ignore]
fn test_spans_simple_tree() {
    init_tree_logger();

    let _s = process_span("outer");

    for i in 0..10 {
        let _s = process_span(format!("middle_{i}"));
        for j in 0..10 {
            let _s = process_span(format!("inner_a_{j}"));
            for k in 0..10 {
                info!(target: "some_target", link_cmd_id = CommandId(k).link(), "At {k}");
            }
        }

        for j2 in 0..10 {
            let _s = process_span(format!("inner_b_{j2}"));
            for k in 0..10 {
                info!(target: "some_target", link_cmd_id = CommandId(k).link(), "At {k}")
            }
        }
    }
}
