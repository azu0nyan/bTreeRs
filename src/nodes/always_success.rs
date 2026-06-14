//! The [`AlwaysSuccess`] leaf.

use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// A leaf that always returns [`Status::Success`]. Mirrors `AlwaysSuccessNode`.
pub struct AlwaysSuccess;

impl<D> BehaviorNode<D> for AlwaysSuccess {
    fn tick(&mut self, _data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        record(dbg, Status::Success, || "AlwaysSuccess".to_string());
        Status::Success
    }
}
