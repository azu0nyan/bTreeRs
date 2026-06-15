//! The [`Wait`] leaf.

use crate::caps::HasDelta;
use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// Reports [`Status::Running`] until `duration` seconds have elapsed, then
/// succeeds. Mirrors `BTWait`.
///
/// Time is accumulated from the context's [`HasDelta::delta_seconds`] each
/// tick; the elapsed total resets after succeeding (and on
/// [`halt`](Wait::halt)) so the node can be reused.
pub struct Wait {
    duration: f64,
    elapsed: f64,
}

impl Wait {
    /// Wait `duration` seconds before succeeding.
    pub fn new(duration: f64) -> Self {
        Self {
            duration,
            elapsed: 0.0,
        }
    }
}

impl<D: HasDelta> BehaviorNode<D> for Wait {
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        self.elapsed += data.delta_seconds();
        let done = self.elapsed >= self.duration;
        let status = if done {
            Status::Success
        } else {
            Status::Running
        };
        record(dbg, status, || {
            format!("Wait {:.3} / {:.3}", self.elapsed, self.duration)
        });
        if done {
            self.elapsed = 0.0;
        }
        status
    }

    fn halt(&mut self) {
        self.elapsed = 0.0;
    }
}
