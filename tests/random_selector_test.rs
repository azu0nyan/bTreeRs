mod common;

use btree::prelude::*;
use common::Sim;

#[test]
fn succeeds_on_first_success() {
    let mut sim = Sim::new(0.0);
    let mut s: RandomSelector<Sim> =
        RandomSelector::new(nodes![AlwaysFailure, AlwaysSuccess, AlwaysFailure]);
    assert_eq!(s.tick(&mut sim, None), Status::Success);
}

#[test]
fn fails_when_all_children_fail() {
    let mut sim = Sim::new(0.0);
    let mut s: RandomSelector<Sim> = RandomSelector::new(nodes![AlwaysFailure, AlwaysFailure]);
    assert_eq!(s.tick(&mut sim, None), Status::Failure);
}
