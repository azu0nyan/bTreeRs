//! The [`AlwaysFailure`] leaf.

use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// A leaf that always returns [`Status::Failure`]. Mirrors `AlwaysFailNode`.
pub struct AlwaysFailure;

impl<D> BehaviorNode<D> for AlwaysFailure {
    fn tick(&mut self, _data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        record(dbg, Status::Failure, || "AlwaysFailure".to_string());
        Status::Failure
    }
}
