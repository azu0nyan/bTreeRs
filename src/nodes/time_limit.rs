//! The [`TimeLimit`] decorator.

use crate::caps::HasDelta;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Fails its child if it runs for longer than `limit` seconds. Mirrors
/// `BTTimeLimit`.
///
/// Elapsed time accumulates from the context's [`HasDelta::delta_seconds`]
/// while the child runs. If the child finishes within the limit its result is
/// returned (and the timer resets); if the limit is exceeded first the child is
/// halted and the node returns [`Status::Failure`].
pub struct TimeLimit<D> {
    child: BoxNode<D>,
    limit: f64,
    elapsed: f64,
}

impl<D> TimeLimit<D> {
    /// Wrap `child` so it fails if it runs longer than `limit` seconds.
    pub fn new(child: impl BehaviorNode<D> + 'static, limit: f64) -> Self {
        Self {
            child: Box::new(child),
            limit,
            elapsed: 0.0,
        }
    }

    fn reset(&mut self) {
        self.child.halt();
        self.elapsed = 0.0;
    }
}

impl<D: HasDelta> BehaviorNode<D> for TimeLimit<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        self.elapsed += data.delta_seconds();
        if self.elapsed > self.limit {
            self.reset();
            record(dbg, Status::Failure, || "TimeLimit (expired)".to_string());
            return Status::Failure;
        }
        let status = tick_child(&mut *self.child, data, &mut dbg);
        if status.is_done() {
            self.reset();
        }
        record(dbg, status, || "TimeLimit".to_string());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        format!("TimeLimit {:.3} / {:.3}", self.elapsed, self.limit)
    }
}
