mod common;

use btree::prelude::*;
use common::{sim_tagged, Sim};

#[test]
fn runs_all_children_when_they_succeed() {
    let mut sim = Sim::new(0.0);
    let mut s = RandomSequence::new(nodes![
        sim_tagged("a", Status::Success),
        sim_tagged("b", Status::Success),
        sim_tagged("c", Status::Success),
    ]);
    assert_eq!(s.tick(&mut sim, None), Status::Success);
    // All three ran exactly once (in some shuffled order).
    let mut ran = sim.log.clone();
    ran.sort_unstable();
    assert_eq!(ran, vec!["a", "b", "c"]);
}

#[test]
fn fails_if_any_child_fails() {
    let mut sim = Sim::new(0.0);
    let mut s: RandomSequence<Sim> = RandomSequence::new(nodes![AlwaysSuccess, AlwaysFailure]);
    assert_eq!(s.tick(&mut sim, None), Status::Failure);
}
