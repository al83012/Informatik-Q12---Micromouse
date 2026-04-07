use tracing::info;

use crate::{logging::run_test, tests::test_map};


// #[test]
// pub fn do_random_moves() -> ! {
//     loop {
//         let world = 
//     }
// }

#[test]
fn test_gen() {
    run_test("debug", || {
        let rand_map = test_map(0.3);
        info!(target: "tests/map/gen", "Finished building map: \n{rand_map}");
    })
}
