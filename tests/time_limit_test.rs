mod common;

use btree::prelude::*;
use common::Sim;

#[test]
fn passes_child_result_within_limit() {
    let mut sim = Sim::new(0.5);
    let mut t = TimeLimit::new(AlwaysSuccess, 1.0);
    assert_eq!(t.tick(&mut sim, None), Status::Success);
}

#[test]
fn fails_when_limit_exceeded() {
    let mut sim = Sim::new(0.6);
    let mut t = TimeLimit::new(AlwaysRunning, 1.0);
    assert_eq!(t.tick(&mut sim, None), Status::Running); // 0.6s
    assert_eq!(t.tick(&mut sim, None), Status::Failure); // 1.2s > 1.0s limit
}
