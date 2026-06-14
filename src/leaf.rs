//! Leaf nodes: the nodes that actually look at and act on the context.
//!
//! * [`Predicate`] — succeeds/fails based on a boolean test.
//! * [`Action`] — runs a closure that returns a [`Status`] directly.
//! * [`AlwaysSuccess`], [`AlwaysFailure`], [`AlwaysRunning`] — constant leaves.

use crate::{BehaviorNode, Status};

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
/// assert_eq!(hungry.tick(&mut hp), Status::Success);
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
    fn tick(&mut self, data: &mut D) -> Status {
        self.last_result = (self.predicate)(data);
        if self.last_result {
            Status::Success
        } else {
            Status::Failure
        }
    }

    fn node_info(&self) -> String {
        match &self.label {
            Some(l) => format!("Predicate: {l} : {}", self.last_result),
            None => format!("Predicate : {}", self.last_result),
        }
    }
}

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
/// assert_eq!(step.tick(&mut c), Status::Running);
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
    fn tick(&mut self, data: &mut D) -> Status {
        (self.action)(data)
    }

    fn node_info(&self) -> String {
        match &self.label {
            Some(l) => format!("Action: {l}"),
            None => "Action".to_string(),
        }
    }
}

/// A leaf that always returns [`Status::Success`]. Mirrors `AlwaysSuccessNode`.
pub struct AlwaysSuccess;

impl<D> BehaviorNode<D> for AlwaysSuccess {
    fn tick(&mut self, _data: &mut D) -> Status {
        Status::Success
    }
}

/// A leaf that always returns [`Status::Failure`]. Mirrors `AlwaysFailNode`.
pub struct AlwaysFailure;

impl<D> BehaviorNode<D> for AlwaysFailure {
    fn tick(&mut self, _data: &mut D) -> Status {
        Status::Failure
    }
}

/// A leaf that always returns [`Status::Running`]. Mirrors `AlwaysRunningNode`.
pub struct AlwaysRunning;

impl<D> BehaviorNode<D> for AlwaysRunning {
    fn tick(&mut self, _data: &mut D) -> Status {
        Status::Running
    }
}
