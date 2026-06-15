//! The [`RunCustomFunc`] leaf.

use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// A leaf that runs a plain function `fn(&mut D) -> Status` and returns its
/// result.
///
/// This is the lightweight, non-capturing sibling of [`Action`](crate::Action):
/// where `Action` boxes a `FnMut` closure (and can capture state),
/// `RunCustomFunc` stores a bare function pointer, so it allocates nothing.
/// Reach for it when your logic is a free function or a non-capturing closure;
/// use `Action` when you need to capture environment.
///
/// ```
/// use btree::prelude::*;
///
/// fn step(counter: &mut i32) -> Status {
///     *counter += 1;
///     if *counter >= 3 { Status::Success } else { Status::Running }
/// }
///
/// let mut node = RunCustomFunc::new(step);
/// let mut c = 0;
/// assert_eq!(node.tick(&mut c, None), Status::Running);
/// assert_eq!(node.tick(&mut c, None), Status::Running);
/// assert_eq!(node.tick(&mut c, None), Status::Success);
/// ```
pub struct RunCustomFunc<D> {
    func: fn(&mut D) -> Status,
}

impl<D> RunCustomFunc<D> {
    /// Build a node that calls `func` on each tick.
    pub fn new(func: fn(&mut D) -> Status) -> Self {
        Self { func }
    }
}

impl<D> BehaviorNode<D> for RunCustomFunc<D> {
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        let status = (self.func)(data);
        record(dbg, status, || "RunCustomFunc".to_string());
        status
    }
}
