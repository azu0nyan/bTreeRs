//! The [`RepeatUntilSuccess`] decorator.

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Re-runs its child until the child succeeds. Mirrors `BTRepeatUntilSuccess`.
///
/// While the child fails, the child is halted and the node reports
/// [`Status::Running`] so it is retried on the next tick; a child
/// [`Status::Running`] is passed through; a child [`Status::Success`] succeeds
/// the node. Unlike [`Retry`](super::Retry) there is no attempt cap — it keeps
/// going until success.
pub struct RepeatUntilSuccess<D> {
    child: BoxNode<D>,
}

impl<D> RepeatUntilSuccess<D> {
    /// Wrap `child`, re-running it until it succeeds.
    pub fn new(child: impl BehaviorNode<D> + 'static) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

impl<D> BehaviorNode<D> for RepeatUntilSuccess<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = match tick_child(&mut *self.child, data, &mut dbg) {
            Status::Success => Status::Success,
            Status::Failure => {
                self.child.halt();
                Status::Running
            }
            Status::Running => Status::Running,
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
    }

    fn node_info(&self) -> String {
        "RepeatUntilSuccess".to_string()
    }
}
