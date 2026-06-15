//! The [`FollowPath`] capability trait.

use crate::Status;

/// A context whose agent can move one step along a previously-computed
/// navigation path.
///
/// Pairs with [`HasNavAgent`](crate::HasNavAgent): once a path exists, the
/// [`GoTo`](crate::GoTo) node drives the agent along it by calling
/// [`advance`](FollowPath::advance) each tick with the frame's delta. Like the
/// other capability traits this keeps the movement node free of any engine
/// dependency — the agent decides what "moving" means.
///
/// ```
/// use btree::prelude::*;
///
/// struct Agent { dist: f64 }
/// impl HasDelta for Agent { fn delta_seconds(&self) -> f64 { 1.0 } }
/// impl FollowPath for Agent {
///     fn advance(&mut self, delta: f64) -> Status {
///         self.dist -= delta;
///         if self.dist <= 0.0 { Status::Success } else { Status::Running }
///     }
/// }
///
/// let mut agent = Agent { dist: 1.5 };
/// let mut go = GoTo::new();
/// assert_eq!(go.tick(&mut agent, None), Status::Running); // 0.5 left
/// assert_eq!(go.tick(&mut agent, None), Status::Success); // arrived
/// ```
pub trait FollowPath {
    /// Advance the agent along its path by `delta` seconds (mirrors a
    /// move-along-path step).
    ///
    /// * [`Status::Running`] — still en route.
    /// * [`Status::Success`] — the destination has been reached.
    /// * [`Status::Failure`] — the path can no longer be followed (e.g. none was
    ///   set, or it became blocked).
    fn advance(&mut self, delta: f64) -> Status;
}
