//! The [`RunLimit`] decorator.

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Lets its child finish at most `limit` times, then fails without ticking it.
/// Mirrors `BTRunLimit`.
///
/// Each time the child *finishes* (returns [`Status::Success`] or
/// [`Status::Failure`]) counts as one run; ticks where the child is still
/// running do not count. Once `limit` runs have completed the node returns
/// [`Status::Failure`] immediately and no longer ticks the child. The counter
/// is *not* cleared by [`halt`](RunLimit::halt) — the cap is for the lifetime
/// of the node.
pub struct RunLimit<D> {
    child: BoxNode<D>,
    limit: u32,
    runs: u32,
}

impl<D> RunLimit<D> {
    /// Wrap `child`, allowing it to finish at most `limit` times.
    pub fn new(child: impl BehaviorNode<D> + 'static, limit: u32) -> Self {
        Self {
            child: Box::new(child),
            limit,
            runs: 0,
        }
    }
}

impl<D> BehaviorNode<D> for RunLimit<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        if self.runs >= self.limit {
            record(dbg, Status::Failure, || self.node_info());
            return Status::Failure;
        }
        let status = tick_child(&mut *self.child, data, &mut dbg);
        if status.is_done() {
            self.runs += 1;
        }
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
    }

    fn node_info(&self) -> String {
        format!("RunLimit {} / {}", self.runs, self.limit)
    }
}
