use btree::prelude::*;

#[test]
fn latches_onto_chosen_branch() {
    // Predicate is true only on the first read; the latching variant must keep
    // ticking the `then` branch it already committed to.
    let mut reads = 0;
    let mut step = 0;
    let mut node = IfThenElse::new(
        move |_: &mut ()| {
            reads += 1;
            reads == 1
        },
        Action::new(move |_: &mut ()| {
            step += 1;
            if step >= 2 { Status::Success } else { Status::Running }
        }),
        AlwaysFailure,
    );
    assert_eq!(node.tick(&mut (), None), Status::Running);
    assert_eq!(node.tick(&mut (), None), Status::Success); // stayed on `then`
}

#[test]
fn picks_else_when_predicate_false() {
    let mut node = IfThenElse::new(|_: &mut ()| false, AlwaysFailure, AlwaysSuccess);
    assert_eq!(node.tick(&mut (), None), Status::Success);
}
