use btree::prelude::*;

#[test]
fn coerces_finished_result_to_failure() {
    let mut n = ForceFailure::new(AlwaysSuccess);
    assert_eq!(n.tick(&mut (), None), Status::Failure);

    let mut n = ForceFailure::new(AlwaysFailure);
    assert_eq!(n.tick(&mut (), None), Status::Failure);
}

#[test]
fn passes_running_through() {
    let mut n = ForceFailure::new(AlwaysRunning);
    assert_eq!(n.tick(&mut (), None), Status::Running);
}
