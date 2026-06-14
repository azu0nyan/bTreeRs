mod common;

use btree::prelude::*;
use common::{tagged, Ctx};

#[test]
fn restarts_from_first_child_every_tick() {
    let mut ctx = Ctx::default();
    let mut seq = ReactiveSequence::new(nodes![
        tagged("a", Status::Success),
        tagged("b", Status::Running),
    ]);
    assert_eq!(seq.tick(&mut ctx, None), Status::Running);
    assert_eq!(seq.tick(&mut ctx, None), Status::Running);
    // "a" is re-evaluated on every tick.
    assert_eq!(ctx.log, vec!["a", "b", "a", "b"]);
}

#[test]
fn fails_on_first_failing_child() {
    let mut ctx = Ctx::default();
    let mut seq = ReactiveSequence::new(nodes![
        tagged("a", Status::Success),
        tagged("b", Status::Failure),
        tagged("c", Status::Success),
    ]);
    assert_eq!(seq.tick(&mut ctx, None), Status::Failure);
    assert_eq!(ctx.log, vec!["a", "b"]);
}
