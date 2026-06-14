mod test_common;

use btree::prelude::*;
use test_common::{tagged, Ctx};

#[test]
fn retries_failed_child_without_resetting() {
    let mut ctx = Ctx::default();
    let mut fail_once = true;
    let mut seq = ProgressiveSequence::new(nodes![
        tagged("a", Status::Success),
        // Fails on its first tick, succeeds afterwards.
        Action::new(move |c: &mut Ctx| {
            c.log.push("b");
            if fail_once {
                fail_once = false;
                Status::Failure
            } else {
                Status::Success
            }
        }),
        tagged("c", Status::Success),
    ]);

    // First tick: "a" ok, "b" fails -> the sequence fails but keeps progress.
    assert_eq!(seq.tick(&mut ctx, None), Status::Failure);
    assert_eq!(ctx.log, vec!["a", "b"]);

    // Second tick: resumes at "b" (does NOT re-run "a"), then runs "c".
    assert_eq!(seq.tick(&mut ctx, None), Status::Success);
    assert_eq!(ctx.log, vec!["a", "b", "b", "c"]);
}
