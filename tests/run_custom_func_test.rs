use btree::prelude::*;

fn increment(c: &mut i32) -> Status {
    *c += 1;
    if *c >= 2 { Status::Success } else { Status::Running }
}

#[test]
fn runs_function_pointer_each_tick() {
    let mut node = RunCustomFunc::new(increment);
    let mut c = 0;
    assert_eq!(node.tick(&mut c, None), Status::Running);
    assert_eq!(node.tick(&mut c, None), Status::Success);
    assert_eq!(c, 2);
}

#[test]
fn accepts_non_capturing_closure() {
    let mut node = RunCustomFunc::new(|_: &mut ()| Status::Failure);
    assert_eq!(node.tick(&mut (), None), Status::Failure);
}

#[test]
fn node_info_is_stable() {
    let node: RunCustomFunc<()> = RunCustomFunc::new(|_| Status::Success);
    assert_eq!(node.node_info(), "RunCustomFunc");
}
