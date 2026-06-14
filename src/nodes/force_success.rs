//! The [`ForceSuccess`] decorator.

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Forces a finished child to [`Status::Success`]. Mirrors `ForceSuccess`.
pub struct ForceSuccess<D> {
    child: BoxNode<D>,
}

impl<D> ForceSuccess<D> {
    /// Wrap `child` so its finished result is always success.
    pub fn new(child: impl BehaviorNode<D> + 'static) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

impl<D> BehaviorNode<D> for ForceSuccess<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = match tick_child(&mut *self.child, data, &mut dbg) {
            Status::Running => Status::Running,
            Status::Success | Status::Failure => Status::Success,
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
    }
}
