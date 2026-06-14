//! The [`FallbackSequence`] composite (fallback / selector with memory).
//!
//! Fallbacks run children left-to-right and stop at the first success (logical
//! OR). This variant remembers its progress between ticks. Compare with
//! [`ReactiveFallbackSequence`](super::ReactiveFallbackSequence), which
//! restarts from the first child every tick.

use super::halt_all;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// A fallback / selector that remembers its progress between ticks.
///
/// Ticks children in order until one succeeds (then the node succeeds and
/// resets) or all fail (then the node fails). A running child returns
/// [`Status::Running`] and progress is kept. Mirrors `FallbackSequence`.
pub struct FallbackSequence<D> {
    children: Vec<BoxNode<D>>,
    current: usize,
}

impl<D> FallbackSequence<D> {
    /// Build a fallback (selector) from its children.
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

impl<D> BehaviorNode<D> for FallbackSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = loop {
            if self.current >= self.children.len() {
                self.reset();
                break Status::Failure;
            }
            match tick_child(&mut *self.children[self.current], data, &mut dbg) {
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
        format!("Fallback {} / {}", self.current, self.children.len())
    }
}
