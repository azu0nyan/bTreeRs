mod common;

use btree::prelude::*;
use common::Sim;

#[test]
fn waits_then_runs_child() {
    let mut sim = Sim::new(0.5);
    let mut d = Delay::new(AlwaysSuccess, 1.0);
    assert_eq!(d.tick(&mut sim, None), Status::Running); // 0.5s < 1.0s, child not run
    assert_eq!(d.tick(&mut sim, None), Status::Success); // delay elapsed, child runs
}
