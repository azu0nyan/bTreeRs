//! The [`IfThenElse`] conditional branch (latching).

use super::{branch_word, Branch};
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Evaluates a predicate, runs the matching branch, and **latches** onto that
/// branch until it finishes.
///
/// On the first tick the predicate selects the `then` or `else` branch. While
/// that branch is running the predicate is *not* re-evaluated; the same branch
/// keeps being ticked until it succeeds or fails, at which point the result is
/// returned and the node resets. Mirrors `IfThenElse`.
pub struct IfThenElse<D> {
    predicate: Box<dyn FnMut(&mut D) -> bool>,
    then_node: BoxNode<D>,
    else_node: BoxNode<D>,
    label: Option<String>,
    running: Option<Branch>,
}

impl<D> IfThenElse<D> {
    /// Build an unlabeled latching if/then/else.
    pub fn new(
        predicate: impl FnMut(&mut D) -> bool + 'static,
        then_node: impl BehaviorNode<D> + 'static,
        else_node: impl BehaviorNode<D> + 'static,
    ) -> Self {
        Self::make(predicate, then_node, else_node, None)
    }

    /// Build a labeled latching if/then/else (the label appears in traces).
    pub fn labeled(
        label: impl Into<String>,
        predicate: impl FnMut(&mut D) -> bool + 'static,
        then_node: impl BehaviorNode<D> + 'static,
        else_node: impl BehaviorNode<D> + 'static,
    ) -> Self {
        Self::make(predicate, then_node, else_node, Some(label.into()))
    }

    fn make(
        predicate: impl FnMut(&mut D) -> bool + 'static,
        then_node: impl BehaviorNode<D> + 'static,
        else_node: impl BehaviorNode<D> + 'static,
        label: Option<String>,
    ) -> Self {
        Self {
            predicate: Box::new(predicate),
            then_node: Box::new(then_node),
            else_node: Box::new(else_node),
            label,
            running: None,
        }
    }
}

impl<D> BehaviorNode<D> for IfThenElse<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let branch = match self.running {
            Some(b) => b,
            None => {
                if (self.predicate)(data) {
                    Branch::Then
                } else {
                    Branch::Else
                }
            }
        };
        let child_status = match branch {
            Branch::Then => tick_child(&mut *self.then_node, data, &mut dbg),
            Branch::Else => tick_child(&mut *self.else_node, data, &mut dbg),
        };
        let status = match child_status {
            Status::Running => {
                self.running = Some(branch);
                Status::Running
            }
            done => {
                self.halt();
                done
            }
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        match self.running {
            Some(Branch::Then) => self.then_node.halt(),
            Some(Branch::Else) => self.else_node.halt(),
            None => {}
        }
        self.running = None;
    }

    fn node_info(&self) -> String {
        format!(
            "IF {} branch {}",
            self.label.as_deref().unwrap_or(""),
            branch_word(self.running)
        )
    }
}
