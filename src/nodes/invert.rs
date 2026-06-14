//! The [`Invert`] decorator.

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Inverts a child's finished result: success becomes failure and vice versa.
/// [`Status::Running`] is passed through unchanged. Mirrors `Invert`.
pub struct Invert<D> {
    child: BoxNode<D>,
}

impl<D> Invert<D> {
    /// Wrap `child` in an inverting decorator.
    pub fn new(child: impl BehaviorNode<D> + 'static) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

impl<D> BehaviorNode<D> for Invert<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = match tick_child(&mut *self.child, data, &mut dbg) {
            Status::Running => Status::Running,
            Status::Success => Status::Failure,
            Status::Failure => Status::Success,
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
    }
}
