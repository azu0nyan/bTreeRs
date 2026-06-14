use btree::prelude::*;

#[test]
fn builds_a_fresh_child_for_each_run() {
    let mut produced = 0;
    let mut p: Producer<i32> = Producer::new(|c: &mut i32| {
        *c += 1;
        AlwaysSuccess.boxed()
    });
    assert_eq!(p.tick(&mut produced, None), Status::Success);
    assert_eq!(p.tick(&mut produced, None), Status::Success);
    assert_eq!(produced, 2); // produced once per run, not cached
}

#[test]
fn keeps_running_child_until_it_finishes() {
    // The produced child runs for two ticks; the producer must keep ticking the
    // same instance instead of rebuilding it.
    let mut p: Producer<i32> = Producer::new(|_: &mut i32| {
        let mut step = 0;
        Action::new(move |_: &mut i32| {
            step += 1;
            if step >= 2 { Status::Success } else { Status::Running }
        })
        .boxed()
    });
    let mut ctx = 0;
    assert_eq!(p.tick(&mut ctx, None), Status::Running);
    assert_eq!(p.tick(&mut ctx, None), Status::Success);
}
