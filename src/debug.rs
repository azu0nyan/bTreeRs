//! Optional debug tracing for behavior trees.
//!
//! Calling [`BehaviorNode::tick`](crate::BehaviorNode::tick) is the normal,
//! zero-overhead way to run a tree. When you want to *see* what happened,
//! call [`BehaviorNode::tick_debug`](crate::BehaviorNode::tick_debug) instead:
//! it performs exactly the same work but also returns a [`DebugNode`] tree
//! describing every node that was processed this tick and the [`Status`] it
//! returned.
//!
//! Tracing is therefore fully opt-in and pay-as-you-go: the plain `tick` path
//! never allocates a [`DebugNode`], so production ticking is unaffected.

use crate::Status;
use std::fmt;

/// A node in a debug trace: the snapshot of one ticked behavior node.
///
/// Returned by [`BehaviorNode::tick_debug`](crate::BehaviorNode::tick_debug).
/// `children` holds the traces of the children that were actually processed
/// during this tick, in the order they were ticked (a node that was skipped or
/// only halted does not appear).
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
    /// Build a childless trace node (for leaves).
    pub fn leaf(name: impl Into<String>, status: Status) -> Self {
        Self {
            name: name.into(),
            status,
            children: Vec::new(),
        }
    }

    /// Build a trace node with children.
    pub fn new(name: impl Into<String>, status: Status, children: Vec<DebugNode>) -> Self {
        Self {
            name: name.into(),
            status,
            children,
        }
    }

    /// Iterate over this node and all of its descendants, depth-first.
    pub fn iter(&self) -> impl Iterator<Item = &DebugNode> {
        // A small explicit stack keeps this allocation-light and avoids
        // recursion in the iterator.
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
