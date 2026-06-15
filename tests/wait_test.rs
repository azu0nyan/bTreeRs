mod common;

use btree::prelude::*;
use common::Sim;

#[test]
fn succeeds_after_duration_elapses() {
    let mut sim = Sim::new(0.5);
    let mut w = Wait::new(1.0);
    assert_eq!(w.tick(&mut sim, None), Status::Running); // 0.5s
    assert_eq!(w.tick(&mut sim, None), Status::Success); // 1.0s
    // Resets so it can be reused.
    assert_eq!(w.tick(&mut sim, None), Status::Running);
}
