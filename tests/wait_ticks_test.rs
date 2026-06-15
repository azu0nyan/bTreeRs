use btree::prelude::*;

#[test]
fn succeeds_on_nth_tick_then_resets() {
    let mut w = WaitTicks::new(3);
    assert_eq!(w.tick(&mut (), None), Status::Running);
    assert_eq!(w.tick(&mut (), None), Status::Running);
    assert_eq!(w.tick(&mut (), None), Status::Success);
    // Resets so it can be reused.
    assert_eq!(w.tick(&mut (), None), Status::Running);
}
