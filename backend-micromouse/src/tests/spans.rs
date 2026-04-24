use tracing::info;

use crate::utils::hyperlink_logging::{init_tree_logger, process_span};

#[test]
#[ignore]
fn test_spans_simple_tree() {
    init_tree_logger();

    let _s = process_span("outer");

    for i in 0..10 {
        let _s = process_span(format!("middle_{i}"));
        for j in 0..10 {
            let _s = process_span(format!("inner_{j}"));
            for k in 0..10 {
                info!(target: "some_target", "At {k}");
            }
        }
    }
}
