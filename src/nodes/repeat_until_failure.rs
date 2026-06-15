//! The [`RepeatUntilFailure`] decorator.

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Re-runs its child until the child fails. Mirrors `BTRepeatUntilFailure`.
///
/// While the child succeeds, the child is halted and the node reports
/// [`Status::Running`] so it is re-run on the next tick; a child
/// [`Status::Running`] is passed through; a child [`Status::Failure`] succeeds
/// the *decorator* (the awaited failure happened), returning
/// [`Status::Failure`].
pub struct RepeatUntilFailure<D> {
    child: BoxNode<D>,
}

impl<D> RepeatUntilFailure<D> {
    /// Wrap `child`, re-running it until it fails.
    pub fn new(child: impl BehaviorNode<D> + 'static) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

impl<D> BehaviorNode<D> for RepeatUntilFailure<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = match tick_child(&mut *self.child, data, &mut dbg) {
            Status::Failure => Status::Failure,
            Status::Success => {
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
        "RepeatUntilFailure".to_string()
    }
}
