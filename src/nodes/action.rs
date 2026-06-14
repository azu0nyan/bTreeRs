//! The [`Action`] leaf.

use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// A leaf that runs a closure returning a [`Status`] directly.
///
/// This is the most general leaf and the natural place to put "do something to
/// the world" logic (move, attack, play an animation, …). It has no Scala
/// counterpart but is the canonical behavior-tree action leaf and the main
/// integration point for an engine such as Godot.
///
/// ```
/// use btree::prelude::*;
///
/// let mut step = Action::new(|counter: &mut i32| {
///     *counter += 1;
///     if *counter >= 3 { Status::Success } else { Status::Running }
/// });
/// let mut c = 0;
/// assert_eq!(step.tick(&mut c, None), Status::Running);
/// ```
pub struct Action<D> {
    action: Box<dyn FnMut(&mut D) -> Status>,
    label: Option<String>,
}

impl<D> Action<D> {
    /// Create an unlabeled action.
    pub fn new(action: impl FnMut(&mut D) -> Status + 'static) -> Self {
        Self {
            action: Box::new(action),
            label: None,
        }
    }

    /// Create an action with a label shown in debug traces.
    pub fn labeled(label: impl Into<String>, action: impl FnMut(&mut D) -> Status + 'static) -> Self {
        Self {
            action: Box::new(action),
            label: Some(label.into()),
        }
    }
}

impl<D> BehaviorNode<D> for Action<D> {
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        let status = (self.action)(data);
        record(dbg, status, || self.node_info());
        status
    }

    fn node_info(&self) -> String {
        match &self.label {
            Some(l) => format!("Action: {l}"),
            None => "Action".to_string(),
        }
    }
}
