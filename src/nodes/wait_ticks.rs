//! The [`WaitTicks`] leaf.

use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// Reports [`Status::Running`] for a number of ticks, then succeeds. Mirrors
/// `BTWaitTicks`.
///
/// Counts behavior-tree ticks (not wall-clock time), so it needs nothing from
/// the context. `WaitTicks::new(n)` succeeds on the `n`-th tick (running on the
/// preceding `n - 1`); the counter resets after succeeding so the node can be
/// reused.
pub struct WaitTicks {
    ticks: u32,
    elapsed: u32,
}

impl WaitTicks {
    /// Wait `ticks` ticks before succeeding.
    pub fn new(ticks: u32) -> Self {
        Self { ticks, elapsed: 0 }
    }
}

impl<D> BehaviorNode<D> for WaitTicks {
    fn tick(&mut self, _data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        self.elapsed += 1;
        let done = self.elapsed >= self.ticks;
        let status = if done {
            Status::Success
        } else {
            Status::Running
        };
        record(dbg, status, || {
            format!("WaitTicks {} / {}", self.elapsed, self.ticks)
        });
        if done {
            self.elapsed = 0;
        }
        status
    }

    fn halt(&mut self) {
        self.elapsed = 0;
    }
}
