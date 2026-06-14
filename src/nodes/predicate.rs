//! The [`Predicate`] leaf.

use crate::debug::record;
use crate::{BehaviorNode, DebugNode, Status};

/// A leaf that succeeds when its predicate is `true` and fails otherwise.
///
/// Mirrors the Scala `PredicateNode`. The closure receives the context so it
/// can query the agent/blackboard; ignore the argument if you only need to
/// read captured state.
///
/// ```
/// use btree::prelude::*;
///
/// let mut hungry = Predicate::labeled("hungry", |hp: &mut i32| *hp < 50);
/// let mut hp = 30;
/// assert_eq!(hungry.tick(&mut hp, None), Status::Success);
/// ```
pub struct Predicate<D> {
    predicate: Box<dyn FnMut(&mut D) -> bool>,
    label: Option<String>,
    last_result: bool,
}

impl<D> Predicate<D> {
    /// Create an unlabeled predicate.
    pub fn new(predicate: impl FnMut(&mut D) -> bool + 'static) -> Self {
        Self {
            predicate: Box::new(predicate),
            label: None,
            last_result: true,
        }
    }

    /// Create a predicate with a label shown in debug traces.
    pub fn labeled(label: impl Into<String>, predicate: impl FnMut(&mut D) -> bool + 'static) -> Self {
        Self {
            predicate: Box::new(predicate),
            label: Some(label.into()),
            last_result: true,
        }
    }
}

impl<D> BehaviorNode<D> for Predicate<D> {
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        self.last_result = (self.predicate)(data);
        let status = if self.last_result {
            Status::Success
        } else {
            Status::Failure
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn node_info(&self) -> String {
        match &self.label {
            Some(l) => format!("Predicate: {l} : {}", self.last_result),
            None => format!("Predicate : {}", self.last_result),
        }
    }
}
