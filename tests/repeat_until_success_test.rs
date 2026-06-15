use btree::prelude::*;

#[test]
fn runs_until_child_succeeds() {
    let mut tries = 0;
    let mut n = RepeatUntilSuccess::new(Action::new(move |_: &mut ()| {
        tries += 1;
        if tries >= 3 {
            Status::Success
        } else {
            Status::Failure
        }
    }));
    assert_eq!(n.tick(&mut (), None), Status::Running); // failed attempt 1
    assert_eq!(n.tick(&mut (), None), Status::Running); // failed attempt 2
    assert_eq!(n.tick(&mut (), None), Status::Success); // succeeds
}

#[test]
fn passes_running_through() {
    let mut n = RepeatUntilSuccess::new(AlwaysRunning);
    assert_eq!(n.tick(&mut (), None), Status::Running);
}
