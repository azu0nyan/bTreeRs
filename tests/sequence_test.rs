mod common;

use btree::prelude::*;
use common::{tagged, Ctx};

#[test]
fn runs_in_order_and_stops_on_failure() {
    let mut ctx = Ctx::default();
    let mut seq = Sequence::new(nodes![
        tagged("a", Status::Success),
        tagged("b", Status::Failure),
        tagged("c", Status::Success),
    ]);
    assert_eq!(seq.tick(&mut ctx, None), Status::Failure);
    assert_eq!(ctx.log, vec!["a", "b"]); // "c" never runs
}

#[test]
fn keeps_progress_across_ticks() {
    let mut ctx = Ctx::default();
    let mut step = 0;
    let mut seq = Sequence::new(nodes![
        tagged("a", Status::Success),
        // Runs once (Running), then succeeds on the next tick.
        Action::new(move |c: &mut Ctx| {
            c.log.push("b");
            step += 1;
            if step >= 2 { Status::Success } else { Status::Running }
        }),
        tagged("c", Status::Success),
    ]);

    assert_eq!(seq.tick(&mut ctx, None), Status::Running);
    // Next tick resumes at "b", not re-running "a".
    assert_eq!(seq.tick(&mut ctx, None), Status::Success);
    assert_eq!(ctx.log, vec!["a", "b", "b", "c"]);
}
