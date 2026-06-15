//! The [`Delay`] decorator.

use crate::caps::HasDelta;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Waits `duration` seconds before it begins ticking its child, then runs the
/// child and returns its result. Mirrors `BTDelay`.
///
/// During the delay the node reports [`Status::Running`]. Once the delay has
/// elapsed the child is ticked every tick; when the child finishes the node
/// returns that result and resets, so a later re-entry waits again. Time comes
/// from the context's [`HasDelta::delta_seconds`].
pub struct Delay<D> {
    child: BoxNode<D>,
    duration: f64,
    elapsed: f64,
    fired: bool,
}

impl<D> Delay<D> {
    /// Delay `child` by `duration` seconds.
    pub fn new(child: impl BehaviorNode<D> + 'static, duration: f64) -> Self {
        Self {
            child: Box::new(child),
            duration,
            elapsed: 0.0,
            fired: false,
        }
    }

    fn reset(&mut self) {
        self.child.halt();
        self.elapsed = 0.0;
        self.fired = false;
    }
}

impl<D: HasDelta> BehaviorNode<D> for Delay<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        if !self.fired {
            self.elapsed += data.delta_seconds();
            if self.elapsed < self.duration {
                let elapsed = self.elapsed;
                let duration = self.duration;
                record(dbg, Status::Running, || {
                    format!("Delay {elapsed:.3} / {duration:.3}")
                });
                return Status::Running;
            }
            self.fired = true;
        }
        let status = tick_child(&mut *self.child, data, &mut dbg);
        if status.is_done() {
            self.reset();
        }
        record(dbg, status, || "Delay (elapsed)".to_string());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        if self.fired {
            "Delay (elapsed)".to_string()
        } else {
            format!("Delay {:.3} / {:.3}", self.elapsed, self.duration)
        }
    }
}
