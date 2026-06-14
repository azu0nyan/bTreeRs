use btree::prelude::*;

#[test]
fn always_returns_success() {
    let mut n = AlwaysSuccess;
    assert_eq!(n.tick(&mut (), None), Status::Success);
    // Works for any context type.
    let mut c = 5;
    assert_eq!(n.tick(&mut c, None), Status::Success);
}
