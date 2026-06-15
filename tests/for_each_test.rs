use btree::prelude::*;

struct It {
    queue: Vec<i32>,
    idx: usize,
    seen: Vec<i32>,
}

#[test]
fn runs_child_for_each_item() {
    let mut ctx = It {
        queue: vec![1, 2, 3],
        idx: 0,
        seen: Vec::new(),
    };
    let mut fe = ForEach::new(
        Action::new(|c: &mut It| {
            let v = c.queue[c.idx - 1];
            c.seen.push(v);
            Status::Success
        }),
        |c: &mut It| {
            if c.idx < c.queue.len() {
                c.idx += 1;
                true
            } else {
                false
            }
        },
    );
    assert_eq!(fe.tick(&mut ctx, None), Status::Success);
    assert_eq!(ctx.seen, vec![1, 2, 3]);
}

#[test]
fn succeeds_immediately_when_empty() {
    let mut fe: ForEach<i32> = ForEach::new(AlwaysSuccess, |_: &mut i32| false);
    assert_eq!(fe.tick(&mut 0, None), Status::Success);
}

#[test]
fn fails_if_child_fails() {
    let mut fe: ForEach<i32> = ForEach::new(AlwaysFailure, |c: &mut i32| {
        *c += 1;
        *c <= 3
    });
    assert_eq!(fe.tick(&mut 0, None), Status::Failure);
}
