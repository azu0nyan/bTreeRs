use btree::prelude::*;

#[test]
fn succeeds_when_all_children_succeed() {
    let mut p: Parallel<()> = Parallel::new(nodes![AlwaysSuccess, AlwaysSuccess]);
    assert_eq!(p.tick(&mut (), None), Status::Success);
}

#[test]
fn fails_as_soon_as_one_child_fails_by_default() {
    let mut p: Parallel<()> = Parallel::new(nodes![AlwaysSuccess, AlwaysFailure]);
    assert_eq!(p.tick(&mut (), None), Status::Failure);
}

#[test]
fn runs_until_every_child_is_done() {
    let mut step = 0;
    let mut p: Parallel<()> = Parallel::new(nodes![
        AlwaysSuccess,
        Action::new(move |_: &mut ()| {
            step += 1;
            if step >= 2 {
                Status::Success
            } else {
                Status::Running
            }
        }),
    ]);
    assert_eq!(p.tick(&mut (), None), Status::Running);
    assert_eq!(p.tick(&mut (), None), Status::Success);
}

#[test]
fn require_one_success_policy() {
    let mut p: Parallel<()> = Parallel::with_policies(
        nodes![AlwaysFailure, AlwaysSuccess],
        ParallelPolicy::RequireOne, // succeed as soon as one succeeds
        ParallelPolicy::RequireAll, // fail only when all fail
    );
    assert_eq!(p.tick(&mut (), None), Status::Success);
}
