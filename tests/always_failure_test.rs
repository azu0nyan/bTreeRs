use btree::prelude::*;

#[test]
fn always_returns_failure() {
    let mut n = AlwaysFailure;
    assert_eq!(n.tick(&mut (), None), Status::Failure);
    let mut c = 5;
    assert_eq!(n.tick(&mut c, None), Status::Failure);
}
