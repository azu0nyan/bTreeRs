use btree::prelude::*;

#[test]
fn runs_child_until_count_successes_then_resets() {
    let mut count = 0;
    let mut r = Repeat::new(
        Action::new(|c: &mut i32| {
            *c += 1;
            Status::Success
        }),
        3,
    );
    assert_eq!(r.tick(&mut count, None), Status::Success);
    assert_eq!(count, 3);
    // Resets and can run the full count again.
    assert_eq!(r.tick(&mut count, None), Status::Success);
    assert_eq!(count, 6);
}

#[test]
fn aborts_on_child_failure() {
    let mut r = Repeat::new(AlwaysFailure, 3);
    assert_eq!(r.tick(&mut (), None), Status::Failure);
}
