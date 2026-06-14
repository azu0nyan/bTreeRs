//! Decorator nodes: single-child nodes that transform their child's result.
//!
//! * [`Invert`] — swaps success and failure.
//! * [`ForceSuccess`] / [`ForceFailure`] — coerce the finished result.
//! * [`Repeat`] — re-run a child up to N successful times.
//! * [`Retry`] — re-run a failing child up to N times.

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

/// Forces a finished child to [`Status::Failure`]. Mirrors `ForceFailure`.
pub struct ForceFailure<D> {
    child: BoxNode<D>,
}

impl<D> ForceFailure<D> {
    /// Wrap `child` so its finished result is always failure.
    pub fn new(child: impl BehaviorNode<D> + 'static) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

impl<D> BehaviorNode<D> for ForceFailure<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = match tick_child(&mut *self.child, data, &mut dbg) {
            Status::Running => Status::Running,
            Status::Success | Status::Failure => Status::Failure,
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
    }
}

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
