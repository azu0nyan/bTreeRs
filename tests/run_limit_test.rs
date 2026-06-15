use btree::prelude::*;

#[test]
fn fails_after_limit_runs() {
    let mut n = RunLimit::new(AlwaysSuccess, 2);
    assert_eq!(n.tick(&mut (), None), Status::Success); // run 1
    assert_eq!(n.tick(&mut (), None), Status::Success); // run 2
    assert_eq!(n.tick(&mut (), None), Status::Failure); // limit reached
}

#[test]
fn running_ticks_do_not_count() {
    let mut step = 0;
    let mut n = RunLimit::new(
        Action::new(move |_: &mut ()| {
            step += 1;
            if step % 2 == 0 {
                Status::Success
            } else {
                Status::Running
            }
        }),
        1,
    );
    assert_eq!(n.tick(&mut (), None), Status::Running); // does not count
    assert_eq!(n.tick(&mut (), None), Status::Success); // run 1
    assert_eq!(n.tick(&mut (), None), Status::Failure); // limit reached
}
