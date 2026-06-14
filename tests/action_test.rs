use btree::prelude::*;

#[test]
fn returns_closure_status_across_ticks() {
    let mut step = Action::new(|c: &mut i32| {
        *c += 1;
        if *c >= 2 { Status::Success } else { Status::Running }
    });
    let mut c = 0;
    assert_eq!(step.tick(&mut c, None), Status::Running);
    assert_eq!(step.tick(&mut c, None), Status::Success);
    assert_eq!(c, 2);
}

#[test]
fn label_appears_in_node_info() {
    let a = Action::labeled("go", |_: &mut ()| Status::Success);
    assert_eq!(a.node_info(), "Action: go");
}
