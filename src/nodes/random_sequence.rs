//! The [`RandomSequence`] composite.

use super::{halt_all, shuffled_order};
use crate::caps::HasRng;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Like [`Sequence`](super::Sequence) (logical AND, with memory) but ticks its
/// children in a freshly shuffled order each run. Mirrors `BTRandomSequence`.
///
/// On (re)entry a random order is drawn from the context's [`HasRng`]. Children
/// are then run in that order: a failure aborts and fails the node, a running
/// child keeps progress, and once all have succeeded the node succeeds and
/// resets (so the next run reshuffles).
pub struct RandomSequence<D> {
    children: Vec<BoxNode<D>>,
    order: Vec<usize>,
    current: usize,
}

impl<D> RandomSequence<D> {
    /// Build a random-order sequence from its children.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self {
            children,
            order: Vec::new(),
            current: 0,
        }
    }

    fn reset(&mut self) {
        self.order.clear();
        self.current = 0;
        halt_all(&mut self.children);
    }
}

impl<D: HasRng> BehaviorNode<D> for RandomSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        if self.order.is_empty() && !self.children.is_empty() {
            self.order = shuffled_order(self.children.len(), data);
            self.current = 0;
        }
        let status = loop {
            if self.current >= self.order.len() {
                self.reset();
                break Status::Success;
            }
            let idx = self.order[self.current];
            match tick_child(&mut *self.children[idx], data, &mut dbg) {
                Status::Failure => {
                    self.reset();
                    break Status::Failure;
                }
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
        format!("RandomSequence {} / {}", self.current, self.children.len())
    }
}
