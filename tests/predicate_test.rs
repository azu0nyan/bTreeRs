use btree::prelude::*;

#[test]
fn succeeds_when_true_fails_when_false() {
    let mut p = Predicate::new(|v: &mut i32| *v > 0);
    let mut v = 5;
    assert_eq!(p.tick(&mut v, None), Status::Success);
    v = -1;
    assert_eq!(p.tick(&mut v, None), Status::Failure);
}

#[test]
fn label_and_result_appear_in_node_info() {
    let mut p = Predicate::labeled("positive", |v: &mut i32| *v > 0);
    let mut v = 1;
    p.tick(&mut v, None);
    assert_eq!(p.node_info(), "Predicate: positive : true");
}
