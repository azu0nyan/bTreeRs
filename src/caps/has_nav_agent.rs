//! The [`HasNavAgent`] capability trait.

use crate::Status;

/// A context whose agent can compute a navigation path to a desired target.
///
/// The behavior tree never touches an engine navigation API directly — the
/// agent does, behind this trait, and reports progress as a [`Status`]. This
/// keeps the routing node ([`SetDestination`](crate::SetDestination))
/// engine-agnostic, exactly as [`HasDelta`](crate::HasDelta) keeps the timers
/// engine-agnostic.
///
/// The desired target is part of the agent's own state (stash it into the
/// context before ticking the tree); calling
/// [`set_destination`](HasNavAgent::set_destination) (re)starts pathfinding
/// toward it. Pair this with [`FollowPath`](crate::FollowPath) — once a path is
/// ready, [`GoTo`](crate::GoTo) walks the agent along it.
///
/// ```
/// use btree::prelude::*;
///
/// struct Agent { has_path: bool }
/// impl HasNavAgent for Agent {
///     fn set_destination(&mut self) -> Status {
///         if self.has_path { Status::Success } else { Status::Failure }
///     }
/// }
///
/// let mut agent = Agent { has_path: true };
/// let mut go = SetDestination::new();
/// assert_eq!(go.tick(&mut agent, None), Status::Success);
/// ```
pub trait HasNavAgent {
    /// Begin (or poll) computing a navigation path to the agent's desired
    /// target.
    ///
    /// * [`Status::Failure`] — no path to the target exists.
    /// * [`Status::Running`] — a path is still being computed (e.g. the engine
    ///   navigation map has not synchronized yet).
    /// * [`Status::Success`] — a path has been found and is ready to follow.
    fn set_destination(&mut self) -> Status;
}
