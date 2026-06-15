//! The [`Cooldown`] decorator.

use crate::caps::HasDelta;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Throttles its child so it runs no more often than once per `duration`
/// seconds. Mirrors `BTCooldown`.
///
/// When off cooldown the child is ticked and its result returned; the moment
/// the child *finishes*, a cooldown of `duration` seconds starts. While on
/// cooldown the node returns [`Status::Failure`] without ticking the child. The
/// cooldown timer counts down using the context's [`HasDelta::delta_seconds`].
pub struct Cooldown<D> {
    child: BoxNode<D>,
    duration: f64,
    remaining: f64,
}

impl<D> Cooldown<D> {
    /// Wrap `child` with a `duration`-second cooldown between runs.
    pub fn new(child: impl BehaviorNode<D> + 'static, duration: f64) -> Self {
        Self {
            child: Box::new(child),
            duration,
            remaining: 0.0,
        }
    }
}

impl<D: HasDelta> BehaviorNode<D> for Cooldown<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        if self.remaining > 0.0 {
            self.remaining = (self.remaining - data.delta_seconds()).max(0.0);
            if self.remaining > 0.0 {
                record(dbg, Status::Failure, || self.node_info());
                return Status::Failure;
            }
        }
        let status = tick_child(&mut *self.child, data, &mut dbg);
        if status.is_done() {
            self.remaining = self.duration;
        }
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.child.halt();
    }

    fn node_info(&self) -> String {
        if self.remaining > 0.0 {
            format!("Cooldown {:.3} left", self.remaining)
        } else {
            "Cooldown (ready)".to_string()
        }
    }
}
