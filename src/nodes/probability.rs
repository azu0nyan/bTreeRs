//! The [`Probability`] decorator.

use crate::caps::HasRng;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Runs its child only with probability `p`; otherwise fails. Mirrors
/// `BTProbability`.
///
/// The dice are rolled once per run, on entry, using the context's [`HasRng`].
/// On a passing roll the child is ticked (across as many ticks as it needs) and
/// its result returned; on a failing roll the node returns [`Status::Failure`]
/// without ticking the child. The decision resets once the run finishes.
pub struct Probability<D> {
    child: BoxNode<D>,
    p: f64,
    run: Option<bool>,
}

impl<D> Probability<D> {
    /// Run `child` with probability `p` (in `[0, 1]`).
    pub fn new(child: impl BehaviorNode<D> + 'static, p: f64) -> Self {
        Self {
            child: Box::new(child),
            p,
            run: None,
        }
    }
}

impl<D: HasRng> BehaviorNode<D> for Probability<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        if self.run.is_none() {
            self.run = Some(data.chance(self.p));
        }
        if self.run == Some(false) {
            self.run = None;
            record(dbg, Status::Failure, || "Probability (skipped)".to_string());
            return Status::Failure;
        }
        let status = tick_child(&mut *self.child, data, &mut dbg);
        if status.is_done() {
            self.run = None;
        }
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
        self.run = None;
    }

    fn node_info(&self) -> String {
        format!("Probability {:.2}", self.p)
    }
}
