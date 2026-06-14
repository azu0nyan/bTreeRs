//! The [`ReactiveSequence`] composite.
//!
//! A sequence (logical AND) that re-evaluates from the first child on every
//! tick rather than resuming where it left off. Compare with
//! [`Sequence`](super::Sequence), which keeps its progress between ticks.

use super::halt_all;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// A sequence that re-evaluates from the first child on every tick ("reactive
/// sequence").
///
/// Each tick starts over at the first child. The first running child causes the
/// node to return [`Status::Running`] (after halting any later children that
/// were left running from a previous tick); the first failure restarts and
/// fails. Mirrors `ReactiveSequence`.
pub struct ReactiveSequence<D> {
    children: Vec<BoxNode<D>>,
}

impl<D> ReactiveSequence<D> {
    /// Build a reactive sequence from its children.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self { children }
    }
}

impl<D> BehaviorNode<D> for ReactiveSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let len = self.children.len();
        let mut status = Status::Success;
        let mut completed = true;
        for i in 0..len {
            match tick_child(&mut *self.children[i], data, &mut dbg) {
                Status::Failure => {
                    halt_all(&mut self.children);
                    status = Status::Failure;
                    completed = false;
                    break;
                }
                Status::Running => {
                    // Cancel later children that may still be running from a
                    // previous tick.
                    for j in (i + 1)..len {
                        self.children[j].halt();
                    }
                    status = Status::Running;
                    completed = false;
                    break;
                }
                Status::Success => continue,
            }
        }
        if completed {
            halt_all(&mut self.children);
        }
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        halt_all(&mut self.children);
    }

    fn node_info(&self) -> String {
        format!("ReactiveSequence {}", self.children.len())
    }
}
