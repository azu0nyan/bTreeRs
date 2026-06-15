//! The [`RandomWait`] leaf.

use crate::caps::{HasDelta, HasRng};
use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// Reports [`Status::Running`] for a random duration in `[min, max]` seconds,
/// then succeeds. Mirrors `BTRandomWait`.
///
/// On entry a target duration is drawn from the context's [`HasRng`]; time then
/// accumulates from [`HasDelta::delta_seconds`] until it is reached. The target
/// and elapsed time reset after succeeding so each run picks a fresh duration.
pub struct RandomWait {
    min: f64,
    max: f64,
    target: Option<f64>,
    elapsed: f64,
}

impl RandomWait {
    /// Wait a random duration in `[min, max]` seconds before succeeding.
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            target: None,
            elapsed: 0.0,
        }
    }
}

impl<D: HasDelta + HasRng> BehaviorNode<D> for RandomWait {
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        let target = match self.target {
            Some(t) => t,
            None => {
                let t = self.min + data.next_f64() * (self.max - self.min);
                self.target = Some(t);
                t
            }
        };
        self.elapsed += data.delta_seconds();
        let done = self.elapsed >= target;
        let status = if done {
            Status::Success
        } else {
            Status::Running
        };
        let elapsed = self.elapsed;
        record(dbg, status, || {
            format!("RandomWait {elapsed:.3} / {target:.3}")
        });
        if done {
            self.target = None;
            self.elapsed = 0.0;
        }
        status
    }

    fn halt(&mut self) {
        self.target = None;
        self.elapsed = 0.0;
    }
}
