//! All debug-tracing tests live here.

mod test_common;

use btree::prelude::*;
use test_common::{tagged, Ctx};

#[test]
fn builds_trace_tree() {
    let mut ctx = Ctx::default();
    let mut tree = Sequence::new(nodes![
        Predicate::labeled("ready", |_: &mut Ctx| true),
        FallbackSequence::new(nodes![
            tagged("a", Status::Failure),
            tagged("b", Status::Running),
        ]),
    ]);

    let (status, trace) = tree.tick_traced(&mut ctx);

    // Root reports the same status as a plain tick would.
    assert_eq!(status, Status::Running);
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
fn nests_to_full_depth() {
    // ReactiveSequence
    //   Predicate "ready"
    //   Invert
    //     FallbackSequence
    //       Action "a" (fails)
    //       Sequence
    //         Action "b" (succeeds)
    //         Action "c" (running)
    let mut ctx = Ctx::default();
    let mut tree = ReactiveSequence::new(nodes![
        Predicate::labeled("ready", |_: &mut Ctx| true),
        Invert::new(FallbackSequence::new(nodes![
            tagged("a", Status::Failure),
            Sequence::new(nodes![
                tagged("b", Status::Success),
                tagged("c", Status::Running),
            ]),
        ])),
    ]);

    let (status, trace) = tree.tick_traced(&mut ctx);
    assert_eq!(status, Status::Running);

    // Level 0: root reactive sequence with two processed children.
    assert!(trace.name.starts_with("ReactiveSequence"));
    assert_eq!(trace.children.len(), 2);
    assert_eq!(trace.children[0].name, "Predicate: ready : true");
    assert_eq!(trace.children[0].status, Status::Success);

    // Level 1: the Invert decorator wraps one child.
    let invert = &trace.children[1];
    assert_eq!(invert.name, "Invert");
    assert_eq!(invert.status, Status::Running); // invert of running is running
    assert_eq!(invert.children.len(), 1);

    // Level 2: the fallback, holding a failed leaf and a running sequence.
    let fallback = &invert.children[0];
    assert!(fallback.name.starts_with("Fallback"));
    assert_eq!(fallback.status, Status::Running);
    assert_eq!(fallback.children.len(), 2);
    assert_eq!(fallback.children[0].name, "Action: a");
    assert_eq!(fallback.children[0].status, Status::Failure);

    // Level 3: the nested sequence, resumed at its running child.
    let seq = &fallback.children[1];
    assert!(seq.name.starts_with("Sequence"));
    assert_eq!(seq.status, Status::Running);
    assert_eq!(seq.children.len(), 2);

    // Level 4: the leaves at the bottom of the tree.
    assert_eq!(seq.children[0].name, "Action: b");
    assert_eq!(seq.children[0].status, Status::Success);
    assert_eq!(seq.children[1].name, "Action: c");
    assert_eq!(seq.children[1].status, Status::Running);

    // Depth-first iteration visits all 8 nodes.
    assert_eq!(trace.iter().count(), 8);

    // Display indents two spaces per level; the deepest leaf is 4 levels down.
    let rendered = trace.to_string();
    assert!(
        rendered.contains("        Action: c [Running]"),
        "deepest leaf should be indented 8 spaces:\n{rendered}"
    );
}

#[test]
fn explicit_slot_is_filled() {
    // The debug object can also be passed straight into `tick`.
    let mut tree = Invert::new(AlwaysSuccess);
    let mut slot = DebugNode::empty();
    let status = tree.tick(&mut (), Some(&mut slot));
    assert_eq!(status, Status::Failure);
    assert_eq!(slot.status, Status::Failure);
    assert_eq!(slot.name, "Invert");
    assert_eq!(slot.children.len(), 1);
    assert_eq!(slot.children[0].name, "AlwaysSuccess");
    assert_eq!(slot.children[0].status, Status::Success);
}

#[test]
fn traced_and_untraced_paths_agree() {
    // Two identical trees ticked in lockstep must return the same status,
    // proving the debug path mirrors the non-debug path.
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
        let sa = a.tick(&mut va, None);
        let (sb, _trace) = b.tick_traced(&mut vb);
        assert_eq!(sa, sb);
        assert_eq!(va, vb);
    }
}
