mod common;

use btree::prelude::*;
use common::Sim;

#[test]
fn forwards_pathfinding_status() {
    let mut node = SetDestination::new();
    let mut sim = Sim::new(0.0);

    sim.nav_status = Status::Running;
    assert_eq!(node.tick(&mut sim, None), Status::Running);

    sim.nav_status = Status::Failure;
    assert_eq!(node.tick(&mut sim, None), Status::Failure);

    sim.nav_status = Status::Success;
    assert_eq!(node.tick(&mut sim, None), Status::Success);
}

#[test]
fn appears_in_trace() {
    let mut node = SetDestination::new();
    let mut sim = Sim::new(0.0);
    let (_status, trace) = node.tick_traced(&mut sim);
    assert!(format!("{trace}").contains("SetDestination"));
}
