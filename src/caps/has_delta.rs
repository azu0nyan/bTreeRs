//! The [`HasDelta`] capability trait.

/// A context that can report the elapsed time of the current tick.
///
/// The timing nodes never read a clock themselves; instead the agent stashes
/// the current frame's delta into the context before ticking the tree, and the
/// nodes accumulate [`delta_seconds`](HasDelta::delta_seconds) across ticks.
/// This keeps the library free of any engine or wall-clock dependency while
/// still supporting real-time behavior (in Godot, pass the `delta` argument of
/// `_process` / `_physics_process`).
///
/// ```
/// use btree::prelude::*;
///
/// struct Agent { dt: f64 }
/// impl HasDelta for Agent {
///     fn delta_seconds(&self) -> f64 { self.dt }
/// }
///
/// let mut agent = Agent { dt: 0.5 };
/// let mut wait = Wait::new(1.0);
/// assert_eq!(wait.tick(&mut agent, None), Status::Running); // 0.5s elapsed
/// assert_eq!(wait.tick(&mut agent, None), Status::Success); // 1.0s elapsed
/// ```
pub trait HasDelta {
    /// Seconds elapsed during the current tick.
    fn delta_seconds(&self) -> f64;
}
