use btree::prelude::*;

#[test]
fn retries_until_child_succeeds() {
    let mut tries = 0;
    let mut r = Retry::new(
        Action::new(move |_: &mut ()| {
            tries += 1;
            if tries >= 3 { Status::Success } else { Status::Failure }
        }),
        5,
    );
    assert_eq!(r.tick(&mut (), None), Status::Success);
}

#[test]
fn fails_after_exhausting_retries() {
    let mut r = Retry::new(AlwaysFailure, 2);
    assert_eq!(r.tick(&mut (), None), Status::Failure);
}
