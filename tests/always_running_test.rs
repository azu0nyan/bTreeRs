use btree::prelude::*;

#[test]
fn always_returns_running() {
    let mut n = AlwaysRunning;
    assert_eq!(n.tick(&mut (), None), Status::Running);
    let mut c = 5;
    assert_eq!(n.tick(&mut c, None), Status::Running);
}
