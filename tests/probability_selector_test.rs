mod common;

use btree::prelude::*;
use common::{sim_tagged, Sim};

#[test]
fn low_roll_picks_first_child() {
    // total weight 3.0; roll = 0.0 * 3.0 = 0.0 < 1.0 -> index 0.
    let mut sim = Sim::with_rng(0.0, vec![0.0]);
    let mut p = ProbabilitySelector::new(vec![
        (1.0, sim_tagged("a", Status::Success).boxed()),
        (2.0, sim_tagged("b", Status::Success).boxed()),
    ]);
    assert_eq!(p.tick(&mut sim, None), Status::Success);
    assert_eq!(sim.log, vec!["a"]);
}

#[test]
fn high_roll_picks_later_child() {
    // roll = 0.9 * 3.0 = 2.7; 2.7 >= 1.0 -> 1.7; 1.7 < 2.0 -> index 1.
    let mut sim = Sim::with_rng(0.0, vec![0.9]);
    let mut p = ProbabilitySelector::new(vec![
        (1.0, sim_tagged("a", Status::Success).boxed()),
        (2.0, sim_tagged("b", Status::Success).boxed()),
    ]);
    assert_eq!(p.tick(&mut sim, None), Status::Success);
    assert_eq!(sim.log, vec!["b"]);
}

#[test]
fn fails_when_weights_sum_to_zero() {
    let mut sim = Sim::new(0.0);
    let mut p: ProbabilitySelector<Sim> =
        ProbabilitySelector::new(vec![(0.0, AlwaysSuccess.boxed())]);
    assert_eq!(p.tick(&mut sim, None), Status::Failure);
}
