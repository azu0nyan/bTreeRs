//! The [`Retry`] decorator.

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Retries its child until it succeeds or has failed `count` times.
///
/// If the child succeeds the node succeeds immediately. If it fails `count`
/// times the node fails. While the child is running the node reports
/// [`Status::Running`] and keeps its progress across ticks. Mirrors `Retry`.
pub struct Retry<D> {
    child: BoxNode<D>,
    count: u32,
    retry: u32,
}

impl<D> Retry<D> {
    /// Retry `child` up to `count` failed runs.
    pub fn new(child: impl BehaviorNode<D> + 'static, count: u32) -> Self {
        Self {
            child: Box::new(child),
            count,
            retry: 0,
        }
    }
}

impl<D> BehaviorNode<D> for Retry<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = loop {
            if self.retry >= self.count {
                self.retry = 0;
                break Status::Failure;
            }
            match tick_child(&mut *self.child, data, &mut dbg) {
                Status::Failure => self.retry += 1,
                Status::Success => {
                    self.retry = 0;
                    break Status::Success;
                }
                Status::Running => break Status::Running,
            }
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
        self.retry = 0;
    }

    fn node_info(&self) -> String {
        format!("Retrying {} / {}", self.retry, self.count)
    }
}
