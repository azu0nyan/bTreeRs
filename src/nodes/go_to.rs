//! The [`GoTo`] leaf.

use crate::caps::{FollowPath, HasDelta};
use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// Drives the context's agent along its navigation path, one step per tick.
///
/// Each tick it advances the agent by the current frame's delta (taken from
/// [`HasDelta`]) via [`FollowPath::advance`] and forwards that [`Status`]:
/// [`Running`](Status::Running) while en route, [`Success`](Status::Success) on
/// arrival, [`Failure`](Status::Failure) if the path can no longer be followed.
/// Pair it after [`SetDestination`](crate::SetDestination), which establishes
/// the path.
#[derive(Default)]
pub struct GoTo;

impl GoTo {
    /// Create a `GoTo` node.
    pub fn new() -> Self {
        Self
    }
}

impl<D: FollowPath + HasDelta> BehaviorNode<D> for GoTo {
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        let delta = data.delta_seconds();
        let status = data.advance(delta);
        record(dbg, status, || "GoTo".to_string());
        status
    }
}
