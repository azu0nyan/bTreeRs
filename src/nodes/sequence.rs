//! The [`Sequence`] composite (sequence with memory).
//!
//! Sequences run children left-to-right and stop at the first failure (logical
//! AND). This variant remembers its progress between ticks: a running child is
//! ticked again on the next tick rather than restarting from the first child.
//! Compare with [`ReactiveSequence`](super::ReactiveSequence) (restarts every
//! tick) and [`ProgressiveSequence`](super::ProgressiveSequence) (also keeps a
//! failing child's position).

use super::halt_all;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// A sequence that remembers its progress between ticks ("sequence with
/// memory").
///
/// Ticks children in order. A child failure restarts the whole sequence and
/// returns [`Status::Failure`]; a running child is ticked again on the next
/// tick (progress is kept); when all children succeed the sequence resets and
/// returns [`Status::Success`]. Mirrors the Scala `Sequence`.
///
/// > Note: the Scala original advanced with `tasks.tail` (a bug that re-ran the
/// > second child forever); this port advances correctly by index, matching the
/// > documented "saves progress between ticks" intent.
pub struct Sequence<D> {
    children: Vec<BoxNode<D>>,
    current: usize,
}

impl<D> Sequence<D> {
    /// Build a sequence from its children. Use the [`nodes!`](crate::nodes!)
    /// macro to box heterogeneous node types ergonomically.
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

impl<D> BehaviorNode<D> for Sequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = loop {
            if self.current >= self.children.len() {
                self.reset();
                break Status::Success;
            }
            match tick_child(&mut *self.children[self.current], data, &mut dbg) {
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
        format!("Sequence {} / {}", self.current, self.children.len())
    }
}
