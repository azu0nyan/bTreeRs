use btree::prelude::*;

#[test]
fn swaps_success_and_failure() {
    let mut n = Invert::new(AlwaysSuccess);
    assert_eq!(n.tick(&mut (), None), Status::Failure);

    let mut n = Invert::new(AlwaysFailure);
    assert_eq!(n.tick(&mut (), None), Status::Success);
}

#[test]
fn passes_running_through() {
    let mut n = Invert::new(AlwaysRunning);
    assert_eq!(n.tick(&mut (), None), Status::Running);
}
