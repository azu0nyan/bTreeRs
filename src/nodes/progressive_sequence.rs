//! The [`ProgressiveSequence`] composite (a.k.a. `SequenceStar`).
//!
//! Like [`Sequence`](super::Sequence) it keeps progress across ticks, but a
//! child failure does *not* reset progress — the failing child is ticked again
//! on the next tick.

use super::halt_all;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// A sequence that never re-runs a finished child until the whole sequence
/// completes (also known as `SequenceStar`).
///
/// Like [`Sequence`](super::Sequence) it keeps progress across ticks, but a
/// child failure does *not* reset progress: the failing child is ticked again
/// next tick. Mirrors `ProgressiveSequence`.
pub struct ProgressiveSequence<D> {
    children: Vec<BoxNode<D>>,
    current: usize,
}

impl<D> ProgressiveSequence<D> {
    /// Build a progressive sequence from its children.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self {
            children,
            current: 0,
        }
    }

    fn reset(&mut self) {
        self.current = 0;
        halt_all(&mut self.children);
    }
}

impl<D> BehaviorNode<D> for ProgressiveSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = loop {
            if self.current >= self.children.len() {
                self.reset();
                break Status::Success;
            }
            match tick_child(&mut *self.children[self.current], data, &mut dbg) {
                // No reset: keep progress so the same child is retried.
                Status::Failure => break Status::Failure,
                Status::Running => break Status::Running,
                Status::Success => self.current += 1,
            }
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        format!("Progressive sequence {} / {}", self.current, self.children.len())
    }
}
