//! The [`Repeat`] decorator.

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Repeats its child until it has succeeded `count` times, then succeeds.
///
/// If the child fails, the repeat is aborted and the node fails. While the
/// child is running the node reports [`Status::Running`] and keeps its progress
/// across ticks. Mirrors `Repeat`.
pub struct Repeat<D> {
    child: BoxNode<D>,
    count: u32,
    repeat: u32,
}

impl<D> Repeat<D> {
    /// Repeat `child` up to `count` successful runs.
    pub fn new(child: impl BehaviorNode<D> + 'static, count: u32) -> Self {
        Self {
            child: Box::new(child),
            count,
            repeat: 0,
        }
    }
}

impl<D> BehaviorNode<D> for Repeat<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = loop {
            if self.repeat >= self.count {
                self.repeat = 0;
                break Status::Success;
            }
            match tick_child(&mut *self.child, data, &mut dbg) {
                Status::Success => self.repeat += 1,
                Status::Failure => {
                    self.repeat = 0;
                    break Status::Failure;
                }
                Status::Running => break Status::Running,
            }
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
        self.repeat = 0;
    }

    fn node_info(&self) -> String {
        format!("Repeating {} / {}", self.repeat, self.count)
    }
}
