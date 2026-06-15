mod common;

use btree::prelude::*;
use common::{sim_tagged, Sim};

#[test]
fn runs_child_on_passing_roll() {
    // draw 0.1 < 0.5 -> run the child.
    let mut sim = Sim::with_rng(0.0, vec![0.1]);
    let mut n = Probability::new(sim_tagged("x", Status::Success), 0.5);
    assert_eq!(n.tick(&mut sim, None), Status::Success);
    assert_eq!(sim.log, vec!["x"]);
}

#[test]
fn skips_child_on_failing_roll() {
    // draw 0.9 < 0.5 is false -> skip the child and fail.
    let mut sim = Sim::with_rng(0.0, vec![0.9]);
    let mut n = Probability::new(sim_tagged("x", Status::Success), 0.5);
    assert_eq!(n.tick(&mut sim, None), Status::Failure);
    assert!(sim.log.is_empty());
}
