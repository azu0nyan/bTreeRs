//! The [`RandomSelector`] composite.

use super::{halt_all, shuffled_order};
use crate::caps::HasRng;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Like [`FallbackSequence`](super::FallbackSequence) (logical OR / selector,
/// with memory) but ticks its children in a freshly shuffled order each run.
/// Mirrors `BTRandomSelector`.
///
/// On (re)entry a random order is drawn from the context's [`HasRng`]. Children
/// are then run in that order: the first success succeeds (and resets) the
/// node, a running child keeps progress, and if all children fail the node
/// fails.
pub struct RandomSelector<D> {
    children: Vec<BoxNode<D>>,
    order: Vec<usize>,
    current: usize,
}

impl<D> RandomSelector<D> {
    /// Build a random-order selector from its children.
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

impl<D: HasRng> BehaviorNode<D> for RandomSelector<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        if self.order.is_empty() && !self.children.is_empty() {
            self.order = shuffled_order(self.children.len(), data);
            self.current = 0;
        }
        let status = loop {
            if self.current >= self.order.len() {
                self.reset();
                break Status::Failure;
            }
            let idx = self.order[self.current];
            match tick_child(&mut *self.children[idx], data, &mut dbg) {
                Status::Success => {
                    self.reset();
                    break Status::Success;
                }
                Status::Running => break Status::Running,
                Status::Failure => self.current += 1,
            }
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        format!("RandomSelector {} / {}", self.current, self.children.len())
    }
}
