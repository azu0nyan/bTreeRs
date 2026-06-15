mod common;

use btree::prelude::*;
use common::Sim;

#[test]
fn advances_until_arrival() {
    let mut node = GoTo::new();
    let mut sim = Sim::new(1.0); // 1s of travel per tick
    sim.nav_remaining = 2.5;

    assert_eq!(node.tick(&mut sim, None), Status::Running); // 1.5 left
    assert_eq!(node.tick(&mut sim, None), Status::Running); // 0.5 left
    assert_eq!(node.tick(&mut sim, None), Status::Success); // arrived
}

#[test]
fn succeeds_immediately_with_no_distance() {
    let mut node = GoTo::new();
    let mut sim = Sim::new(0.5);
    // nav_remaining defaults to 0.0 — already there.
    assert_eq!(node.tick(&mut sim, None), Status::Success);
}

#[test]
fn routes_then_travels_in_a_sequence() {
    let mut tree = Sequence::new(nodes![SetDestination::new(), GoTo::new()]);
    let mut sim = Sim::new(1.0);
    sim.nav_status = Status::Success; // path found right away
    sim.nav_remaining = 1.5;

    assert_eq!(tree.tick(&mut sim, None), Status::Running); // routed, 0.5 left
    assert_eq!(tree.tick(&mut sim, None), Status::Success); // arrived
}
