use btree::prelude::*;

#[test]
fn coerces_finished_result_to_success() {
    let mut n = ForceSuccess::new(AlwaysFailure);
    assert_eq!(n.tick(&mut (), None), Status::Success);

    let mut n = ForceSuccess::new(AlwaysSuccess);
    assert_eq!(n.tick(&mut (), None), Status::Success);
}

#[test]
fn passes_running_through() {
    let mut n = ForceSuccess::new(AlwaysRunning);
    assert_eq!(n.tick(&mut (), None), Status::Running);
}
