mod test_common;

use btree::prelude::*;
use test_common::{tagged, Ctx};

#[test]
fn reevaluates_from_first_child_each_tick() {
    let mut ctx = Ctx::default();
    let mut fb = ReactiveFallbackSequence::new(nodes![
        tagged("a", Status::Failure),
        tagged("b", Status::Running),
    ]);
    assert_eq!(fb.tick(&mut ctx, None), Status::Running);
    assert_eq!(fb.tick(&mut ctx, None), Status::Running);
    // "a" is re-evaluated on every tick.
    assert_eq!(ctx.log, vec!["a", "b", "a", "b"]);
}

#[test]
fn first_success_wins() {
    let mut ctx = Ctx::default();
    let mut fb = ReactiveFallbackSequence::new(nodes![
        tagged("a", Status::Failure),
        tagged("b", Status::Success),
        tagged("c", Status::Success),
    ]);
    assert_eq!(fb.tick(&mut ctx, None), Status::Success);
    assert_eq!(ctx.log, vec!["a", "b"]);
}
