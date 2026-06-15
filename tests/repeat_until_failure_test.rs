use btree::prelude::*;

#[test]
fn runs_until_child_fails() {
    let mut tries = 0;
    let mut n = RepeatUntilFailure::new(Action::new(move |_: &mut ()| {
        tries += 1;
        if tries >= 3 {
            Status::Failure
        } else {
            Status::Success
        }
    }));
    assert_eq!(n.tick(&mut (), None), Status::Running); // succeeded, keep going
    assert_eq!(n.tick(&mut (), None), Status::Running);
    assert_eq!(n.tick(&mut (), None), Status::Failure); // the awaited failure
}
