//! Optional debug tracing for behavior trees.
//!
//! Tracing is threaded through the *single* [`tick`](crate::BehaviorNode::tick)
//! method via an `Option<&mut DebugNode>` argument:
//!
//! * Pass `None` (the normal, production path) and nothing is recorded — no
//!   allocation, no string formatting, essentially free.
//! * Pass `Some(&mut slot)` and every node that runs fills its slot with its
//!   name, status, and the slots of the children it processed, building a tree.
//!
//! The convenience [`tick_traced`](crate::BehaviorNode::tick_traced) wraps the
//! `Some` case for you and hands back the finished [`DebugNode`].
//!
//! Custom node implementations record into the optional slot with the
//! [`record`] helper (leaves) and [`tick_child`] helper (composites), which
//! both no-op when tracing is disabled.

use crate::{BehaviorNode, Status};
use std::fmt;

/// A node in a debug trace: a snapshot of one ticked behavior node.
///
/// This is the "debug object" passed into
/// [`tick`](crate::BehaviorNode::tick). When tracing is enabled a node fills in
/// its [`name`](DebugNode::name) and [`status`](DebugNode::status) and pushes a
/// child `DebugNode` for each child it processed this tick, in tick order. A
/// child that was skipped or only halted does not appear.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DebugNode {
    /// Human-readable description of the node (its
    /// [`node_info`](crate::BehaviorNode::node_info)).
    pub name: String,
    /// The status this node returned for the tick.
    pub status: Status,
    /// Traces of the children processed this tick, in tick order.
    pub children: Vec<DebugNode>,
}

impl DebugNode {
    /// An empty slot to be handed to [`tick`](crate::BehaviorNode::tick) and
    /// filled by the node.
    ///
    /// The `status` is a placeholder ([`Status::Failure`]) that every node
    /// overwrites the instant it records into the slot.
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            status: Status::Failure,
            children: Vec::new(),
        }
    }

    /// Iterate over this node and all of its descendants, depth-first.
    pub fn iter(&self) -> impl Iterator<Item = &DebugNode> {
        let mut stack = vec![self];
        std::iter::from_fn(move || {
            let node = stack.pop()?;
            stack.extend(node.children.iter().rev());
            Some(node)
        })
    }

    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        for _ in 0..depth {
            f.write_str("  ")?;
        }
        writeln!(f, "{} [{:?}]", self.name, self.status)?;
        for child in &self.children {
            child.fmt_indented(f, depth + 1)?;
        }
        Ok(())
    }
}

/// Pretty-prints the trace as an indented tree, one node per line, e.g.:
///
/// ```text
/// Sequence 1 / 2 [Running]
///   Predicate: hungry : true [Success]
///   Action: eat [Running]
/// ```
impl fmt::Display for DebugNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
}

/// Record a node's own `name` and `status` into its optional debug slot.
///
/// Used by leaf nodes (and at the end of composite ticks) when implementing
/// [`BehaviorNode::tick`]. The `name` closure is only evaluated when a slot is
/// actually present, so the non-debug path never pays for formatting.
///
/// ```
/// use btree::prelude::*;
///
/// struct MyLeaf;
/// impl BehaviorNode<()> for MyLeaf {
///     fn tick(&mut self, _data: &mut (), dbg: Option<&mut DebugNode>) -> Status {
///         let status = Status::Success;
///         record(dbg, status, || "MyLeaf".to_string());
///         status
///     }
/// }
/// ```
#[inline]
pub fn record(dbg: Option<&mut DebugNode>, status: Status, name: impl FnOnce() -> String) {
    if let Some(slot) = dbg {
        slot.name = name();
        slot.status = status;
    }
}

/// Tick a child, recording its trace into `parent` when tracing is enabled.
///
/// Used by composite nodes when implementing [`BehaviorNode::tick`]: pass the
/// parent's own `Option<&mut DebugNode>` (by `&mut`) and this allocates a child
/// slot, ticks the child into it, and pushes it onto `parent.children`. When
/// `parent` is `None` it simply forwards `None` to the child — no allocation.
#[inline]
pub fn tick_child<D>(
    child: &mut dyn BehaviorNode<D>,
    data: &mut D,
    parent: &mut Option<&mut DebugNode>,
) -> Status {
    match parent.as_deref_mut() {
        Some(slot) => {
            let mut sub = DebugNode::empty();
            let status = child.tick(data, Some(&mut sub));
            slot.children.push(sub);
            status
        }
        None => child.tick(data, None),
    }
}
