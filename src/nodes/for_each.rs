//! The [`ForEach`] composite.

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Closure that advances iteration: stash the next item into the context and
/// return whether one existed.
type Advance<D> = Box<dyn FnMut(&mut D) -> bool>;

/// Runs a single child once per item of a collection sourced from the context.
/// Mirrors `BTForEach`.
///
/// Because the library is generic over the context `D` and never inspects its
/// fields, iteration is driven by a closure rather than a blackboard array: on
/// entry, and after each successful child run, the `advance` closure is called
/// to move to the next item (typically writing it into `D` so the child can
/// read it) and to report whether one was available.
///
/// * the child running over the current item returns [`Status::Running`];
/// * a child [`Status::Failure`] aborts the whole loop with failure;
/// * when `advance` reports no more items, the loop succeeds.
///
/// Items the child completes instantly are consumed within a single tick, just
/// as [`Sequence`](super::Sequence) advances through instant children.
pub struct ForEach<D> {
    child: BoxNode<D>,
    advance: Advance<D>,
    started: bool,
}

impl<D> ForEach<D> {
    /// Build a `ForEach` running `child` for each item produced by `advance`.
    ///
    /// `advance` should move to the next item (e.g. write it into the context)
    /// and return `true` if there was a next item, `false` once exhausted.
    pub fn new(
        child: impl BehaviorNode<D> + 'static,
        advance: impl FnMut(&mut D) -> bool + 'static,
    ) -> Self {
        Self {
            child: Box::new(child),
            advance: Box::new(advance),
            started: false,
        }
    }

    fn reset(&mut self) {
        self.child.halt();
        self.started = false;
    }
}

impl<D> BehaviorNode<D> for ForEach<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        if !self.started {
            self.started = true;
            if !(self.advance)(data) {
                self.reset();
                record(dbg, Status::Success, || "ForEach (empty)".to_string());
                return Status::Success;
            }
        }
        let status = loop {
            match tick_child(&mut *self.child, data, &mut dbg) {
                Status::Running => break Status::Running,
                Status::Failure => {
                    self.reset();
                    break Status::Failure;
                }
                Status::Success => {
                    self.child.halt();
                    if !(self.advance)(data) {
                        self.reset();
                        break Status::Success;
                    }
                }
            }
        };
        record(dbg, status, || "ForEach".to_string());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        "ForEach".to_string()
    }
}
