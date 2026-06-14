//! The [`AlwaysRunning`] leaf.

use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// A leaf that always returns [`Status::Running`]. Mirrors `AlwaysRunningNode`.
pub struct AlwaysRunning;

impl<D> BehaviorNode<D> for AlwaysRunning {
    fn tick(&mut self, _data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        record(dbg, Status::Running, || "AlwaysRunning".to_string());
        Status::Running
    }
}
