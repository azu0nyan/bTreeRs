mod common;

use btree::prelude::*;
use common::Sim;

#[test]
fn waits_a_random_duration() {
    // RNG draw 0.5 -> target = 1.0 + 0.5 * (2.0 - 1.0) = 1.5s.
    let mut sim = Sim::with_rng(0.5, vec![0.5]);
    let mut w = RandomWait::new(1.0, 2.0);
    assert_eq!(w.tick(&mut sim, None), Status::Running); // 0.5s
    assert_eq!(w.tick(&mut sim, None), Status::Running); // 1.0s
    assert_eq!(w.tick(&mut sim, None), Status::Success); // 1.5s
}
