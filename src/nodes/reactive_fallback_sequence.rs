//! The [`ReactiveFallbackSequence`] composite.
//!
//! A fallback / selector (logical OR) that re-evaluates from the first child on
//! every tick rather than resuming where it left off. Compare with
//! [`FallbackSequence`](super::FallbackSequence), which keeps its progress
//! between ticks.

use super::halt_all;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// A fallback / selector that re-evaluates from the first child on every tick.
///
/// Each tick starts over at the first child; the first success wins, the first
/// running child returns [`Status::Running`] (after halting later children),
/// and a failing child is halted before moving on. Mirrors
/// `ReactiveFallbackSequence`.
pub struct ReactiveFallbackSequence<D> {
    children: Vec<BoxNode<D>>,
}

impl<D> ReactiveFallbackSequence<D> {
    /// Build a reactive fallback from its children.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self { children }
    }
}

impl<D> BehaviorNode<D> for ReactiveFallbackSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let len = self.children.len();
        let mut status = Status::Failure;
        let mut completed = true;
        for i in 0..len {
            match tick_child(&mut *self.children[i], data, &mut dbg) {
                Status::Success => {
                    halt_all(&mut self.children);
                    status = Status::Success;
                    completed = false;
                    break;
                }
                Status::Running => {
                    for j in (i + 1)..len {
                        self.children[j].halt();
                    }
                    status = Status::Running;
                    completed = false;
                    break;
                }
                Status::Failure => self.children[i].halt(),
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
        format!("Reactive fallback {}", self.children.len())
    }
}
