//! The [`SetDestination`] leaf.

use crate::caps::HasNavAgent;
use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// Asks the context's navigation agent to compute a path to its desired target,
/// forwarding the agent's pathfinding [`Status`].
///
/// Returns [`Status::Failure`] if no path exists, [`Status::Running`] while the
/// path is still being computed, and [`Status::Success`] once a path is ready
/// to follow (see [`HasNavAgent::set_destination`]). Pair it with
/// [`GoTo`](crate::GoTo) — typically in a [`Sequence`](crate::Sequence) — to
/// route, then travel.
#[derive(Default)]
pub struct SetDestination;

impl SetDestination {
    /// Create a `SetDestination` node.
    pub fn new() -> Self {
        Self
    }
}

impl<D: HasNavAgent> BehaviorNode<D> for SetDestination {
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        let status = data.set_destination();
        record(dbg, status, || "SetDestination".to_string());
        status
    }
}
