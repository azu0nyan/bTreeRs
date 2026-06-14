mod test_common;

use btree::prelude::*;
use test_common::{tagged, Ctx};

#[test]
fn returns_first_success() {
    let mut ctx = Ctx::default();
    let mut fb = FallbackSequence::new(nodes![
        tagged("a", Status::Failure),
        tagged("b", Status::Success),
        tagged("c", Status::Success),
    ]);
    assert_eq!(fb.tick(&mut ctx, None), Status::Success);
    assert_eq!(ctx.log, vec!["a", "b"]); // "c" never runs
}

#[test]
fn fails_when_all_children_fail() {
    let mut ctx = Ctx::default();
    let mut fb = FallbackSequence::new(nodes![
        tagged("a", Status::Failure),
        tagged("b", Status::Failure),
    ]);
    assert_eq!(fb.tick(&mut ctx, None), Status::Failure);
    assert_eq!(ctx.log, vec!["a", "b"]);
}
