//! The [`ReactiveIfThenElse`] conditional branch (re-evaluating).

use super::{branch_word, Branch};
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Re-evaluates its predicate every tick and runs the matching branch,
/// switching branches (and halting the abandoned one) whenever the predicate
/// flips.
///
/// Unlike [`IfThenElse`](super::IfThenElse) this never latches: if the
/// predicate changes while the active branch is running, the running branch is
/// halted and the other branch is ticked instead. Mirrors `ReactiveIfThenElse`.
pub struct ReactiveIfThenElse<D> {
    predicate: Box<dyn FnMut(&mut D) -> bool>,
    then_node: BoxNode<D>,
    else_node: BoxNode<D>,
    label: Option<String>,
    running: Option<Branch>,
}

impl<D> ReactiveIfThenElse<D> {
    /// Build an unlabeled reactive if/then/else.
    pub fn new(
        predicate: impl FnMut(&mut D) -> bool + 'static,
        then_node: impl BehaviorNode<D> + 'static,
        else_node: impl BehaviorNode<D> + 'static,
    ) -> Self {
        Self::make(predicate, then_node, else_node, None)
    }

    /// Build a labeled reactive if/then/else (the label appears in traces).
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

impl<D> BehaviorNode<D> for ReactiveIfThenElse<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = if (self.predicate)(data) {
            // Switched away from the else branch: cancel it.
            if self.running == Some(Branch::Else) {
                self.else_node.halt();
            }
            self.running = None;
            match tick_child(&mut *self.then_node, data, &mut dbg) {
                Status::Running => {
                    self.running = Some(Branch::Then);
                    Status::Running
                }
                done => {
                    self.then_node.halt();
                    done
                }
            }
        } else {
            // Switched away from the then branch: cancel it.
            if self.running == Some(Branch::Then) {
                self.then_node.halt();
            }
            self.running = None;
            match tick_child(&mut *self.else_node, data, &mut dbg) {
                Status::Running => {
                    self.running = Some(Branch::Else);
                    Status::Running
                }
                done => {
                    self.else_node.halt();
                    done
                }
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
            "REACTIVE IF {} branch {}",
            self.label.as_deref().unwrap_or(""),
            branch_word(self.running)
        )
    }
}
