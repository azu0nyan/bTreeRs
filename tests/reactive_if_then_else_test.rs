use btree::prelude::*;

#[test]
fn switches_branch_when_predicate_flips() {
    let mut cond = true;
    let mut node = ReactiveIfThenElse::new(
        |c: &mut bool| *c,
        Action::new(|_: &mut bool| Status::Running),
        Action::new(|_: &mut bool| Status::Success),
    );
    assert_eq!(node.tick(&mut cond, None), Status::Running); // then branch
    cond = false;
    assert_eq!(node.tick(&mut cond, None), Status::Success); // switched to else
}
