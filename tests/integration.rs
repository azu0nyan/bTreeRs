use btree::prelude::*;

/// A tiny context used across the tests.
#[derive(Default)]
struct Ctx {
    log: Vec<&'static str>,
}

/// An action that records a tag and returns a fixed status.
fn tagged(tag: &'static str, status: Status) -> Action<Ctx> {
    Action::new(move |c: &mut Ctx| {
        c.log.push(tag);
        status
    })
}

#[test]
fn predicate_succeeds_and_fails() {
    let mut p = Predicate::new(|v: &mut i32| *v > 0);
    let mut v = 5;
    assert_eq!(p.tick(&mut v), Status::Success);
    v = -1;
    assert_eq!(p.tick(&mut v), Status::Failure);
}

#[test]
fn sequence_runs_in_order_and_stops_on_failure() {
    let mut ctx = Ctx::default();
    let mut seq = Sequence::new(nodes![
        tagged("a", Status::Success),
        tagged("b", Status::Failure),
        tagged("c", Status::Success),
    ]);
    assert_eq!(seq.tick(&mut ctx), Status::Failure);
    assert_eq!(ctx.log, vec!["a", "b"]); // "c" never runs
}

#[test]
fn sequence_keeps_progress_across_ticks() {
    let mut ctx = Ctx::default();
    let mut step = 0;
    let mut seq = Sequence::new(nodes![
        tagged("a", Status::Success),
        // Runs once, then succeeds.
        Action::new(move |c: &mut Ctx| {
            c.log.push("b");
            step += 1;
            if step >= 2 { Status::Success } else { Status::Running }
        }),
        tagged("c", Status::Success),
    ]);

    assert_eq!(seq.tick(&mut ctx), Status::Running);
    // Next tick must resume at "b", not re-run "a".
    assert_eq!(seq.tick(&mut ctx), Status::Success);
    assert_eq!(ctx.log, vec!["a", "b", "b", "c"]);
}

#[test]
fn reactive_sequence_restarts_every_tick() {
    let mut ctx = Ctx::default();
    let mut seq = ReactiveSequence::new(nodes![
        tagged("a", Status::Success),
        tagged("b", Status::Running),
    ]);
    assert_eq!(seq.tick(&mut ctx), Status::Running);
    assert_eq!(seq.tick(&mut ctx), Status::Running);
    // "a" is re-evaluated on every tick.
    assert_eq!(ctx.log, vec!["a", "b", "a", "b"]);
}

#[test]
fn fallback_returns_first_success() {
    let mut ctx = Ctx::default();
    let mut fb = FallbackSequence::new(nodes![
        tagged("a", Status::Failure),
        tagged("b", Status::Success),
        tagged("c", Status::Success),
    ]);
    assert_eq!(fb.tick(&mut ctx), Status::Success);
    assert_eq!(ctx.log, vec!["a", "b"]);
}

#[test]
fn invert_swaps_result() {
    let mut n = Invert::new(AlwaysSuccess);
    assert_eq!(n.tick(&mut ()), Status::Failure);
    let mut n = Invert::new(AlwaysFailure);
    assert_eq!(n.tick(&mut ()), Status::Success);
}

#[test]
fn repeat_runs_n_times() {
    let mut count = 0;
    let mut r = Repeat::new(
        Action::new(|c: &mut i32| {
            *c += 1;
            Status::Success
        }),
        3,
    );
    assert_eq!(r.tick(&mut count), Status::Success);
    assert_eq!(count, 3);
    // After completing, it resets and can run again.
    assert_eq!(r.tick(&mut count), Status::Success);
    assert_eq!(count, 6);
}

#[test]
fn retry_until_success() {
    let mut tries = 0;
    let mut r = Retry::new(
        Action::new(move |_: &mut ()| {
            tries += 1;
            if tries >= 3 { Status::Success } else { Status::Failure }
        }),
        5,
    );
    assert_eq!(r.tick(&mut ()), Status::Success);
}

#[test]
fn retry_exhausts() {
    let mut r = Retry::new(AlwaysFailure, 2);
    assert_eq!(r.tick(&mut ()), Status::Failure);
}

#[test]
fn if_then_else_latches() {
    // Predicate flips to false after the first read, but the latching variant
    // must keep ticking the `then` branch it already committed to.
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
    assert_eq!(node.tick(&mut ()), Status::Running);
    assert_eq!(node.tick(&mut ()), Status::Success); // stayed on `then`
}

#[test]
fn reactive_if_then_else_switches() {
    let mut cond = true;
    // We can't capture `cond` by ref in two closures, so drive via context.
    let mut node = ReactiveIfThenElse::new(
        |c: &mut bool| *c,
        Action::new(|_: &mut bool| Status::Running),
        Action::new(|_: &mut bool| Status::Success),
    );
    assert_eq!(node.tick(&mut cond), Status::Running); // then branch
    cond = false;
    assert_eq!(node.tick(&mut cond), Status::Success); // switched to else
}

#[test]
fn tick_debug_builds_trace_tree() {
    let mut ctx = Ctx::default();
    let mut tree = Sequence::new(nodes![
        Predicate::labeled("ready", |_: &mut Ctx| true),
        FallbackSequence::new(nodes![
            tagged("a", Status::Failure),
            tagged("b", Status::Running),
        ]),
    ]);

    let trace = tree.tick_debug(&mut ctx);

    // Root reports the same status as a plain tick would.
    assert_eq!(trace.status, Status::Running);
    assert!(trace.name.starts_with("Sequence"));

    // The sequence processed both children this tick.
    assert_eq!(trace.children.len(), 2);
    assert_eq!(trace.children[0].name, "Predicate: ready : true");
    assert_eq!(trace.children[0].status, Status::Success);

    // The fallback processed "a" (failed) then "b" (running).
    let fallback = &trace.children[1];
    assert_eq!(fallback.status, Status::Running);
    assert_eq!(fallback.children.len(), 2);
    assert_eq!(fallback.children[0].status, Status::Failure);
    assert_eq!(fallback.children[1].status, Status::Running);

    // The flattened iterator visits every node (1 root + 2 + 2 children).
    assert_eq!(trace.iter().count(), 5);

    // Display renders an indented tree.
    let rendered = trace.to_string();
    assert!(rendered.contains("Sequence"));
    assert!(rendered.contains("[Running]"));
}

#[test]
fn tick_and_tick_debug_agree() {
    // Two identical trees ticked in lockstep must return the same status,
    // proving the debug path mirrors the fast path.
    let build = || {
        ReactiveSequence::new(nodes![
            Predicate::new(|v: &mut i32| *v > 0),
            Action::new(|v: &mut i32| {
                *v -= 1;
                if *v <= 0 { Status::Success } else { Status::Running }
            }),
        ])
    };
    let mut a = build();
    let mut b = build();
    let mut va = 3;
    let mut vb = 3;
    for _ in 0..5 {
        let sa = a.tick(&mut va);
        let sb = b.tick_debug(&mut vb).status;
        assert_eq!(sa, sb);
        assert_eq!(va, vb);
    }
}

#[test]
fn producer_builds_child_lazily() {
    let mut produced = 0;
    let mut p: Producer<i32> = Producer::new(|c: &mut i32| {
        *c += 1;
        AlwaysSuccess.boxed()
    });
    assert_eq!(p.tick(&mut produced), Status::Success);
    assert_eq!(p.tick(&mut produced), Status::Success);
    assert_eq!(produced, 2); // a fresh child is built for each run
}
