mod common;

use btree::prelude::*;
use common::Sim;

#[test]
fn blocks_until_cooldown_elapses() {
    let mut sim = Sim::new(0.5);
    let mut c = Cooldown::new(AlwaysSuccess, 1.0);
    assert_eq!(c.tick(&mut sim, None), Status::Success); // runs, starts a 1.0s cooldown
    assert_eq!(c.tick(&mut sim, None), Status::Failure); // 0.5s remaining, blocked
    assert_eq!(c.tick(&mut sim, None), Status::Success); // cooldown elapsed, runs again
}
