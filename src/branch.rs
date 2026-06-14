//! Conditional branch nodes.
//!
//! * [`IfThenElse`] — picks a branch by predicate and *latches* onto it until
//!   the branch finishes.
//! * [`ReactiveIfThenElse`] — re-checks the predicate every tick and switches
//!   branches when it changes.

use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Which branch of a conditional is currently running.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Branch {
    Then,
    Else,
}

fn branch_word(running: Option<Branch>) -> &'static str {
    match running {
        Some(Branch::Then) => "then",
        Some(Branch::Else) => "else",
        None => "NO BRANCH",
    }
}

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
    fn tick(&mut self, data: &mut D) -> Status {
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
        let status = match branch {
            Branch::Then => self.then_node.tick(data),
            Branch::Else => self.else_node.tick(data),
        };
        match status {
            Status::Running => {
                self.running = Some(branch);
                Status::Running
            }
            done => {
                self.halt();
                done
            }
        }
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

    fn tick_debug(&mut self, data: &mut D) -> DebugNode {
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
        let child = match branch {
            Branch::Then => self.then_node.tick_debug(data),
            Branch::Else => self.else_node.tick_debug(data),
        };
        let status = match child.status {
            Status::Running => {
                self.running = Some(branch);
                Status::Running
            }
            done => {
                self.halt();
                done
            }
        };
        DebugNode::new(self.node_info(), status, vec![child])
    }
}

/// Re-evaluates its predicate every tick and runs the matching branch,
/// switching branches (and halting the abandoned one) whenever the predicate
/// flips.
///
/// Unlike [`IfThenElse`] this never latches: if the predicate changes while the
/// active branch is running, the running branch is halted and the other branch
/// is ticked instead. Mirrors `ReactiveIfThenElse`.
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
    fn tick(&mut self, data: &mut D) -> Status {
        if (self.predicate)(data) {
            // Switched away from the else branch: cancel it.
            if self.running == Some(Branch::Else) {
                self.else_node.halt();
            }
            self.running = None;
            match self.then_node.tick(data) {
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
            match self.else_node.tick(data) {
                Status::Running => {
                    self.running = Some(Branch::Else);
                    Status::Running
                }
                done => {
                    self.else_node.halt();
                    done
                }
            }
        }
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

    fn tick_debug(&mut self, data: &mut D) -> DebugNode {
        let (status, child) = if (self.predicate)(data) {
            if self.running == Some(Branch::Else) {
                self.else_node.halt();
            }
            self.running = None;
            let child = self.then_node.tick_debug(data);
            let status = match child.status {
                Status::Running => {
                    self.running = Some(Branch::Then);
                    Status::Running
                }
                done => {
                    self.then_node.halt();
                    done
                }
            };
            (status, child)
        } else {
            if self.running == Some(Branch::Then) {
                self.then_node.halt();
            }
            self.running = None;
            let child = self.else_node.tick_debug(data);
            let status = match child.status {
                Status::Running => {
                    self.running = Some(Branch::Else);
                    Status::Running
                }
                done => {
                    self.else_node.halt();
                    done
                }
            };
            (status, child)
        };
        DebugNode::new(self.node_info(), status, vec![child])
    }
}
