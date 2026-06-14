//! The [`Producer`] node, which builds its child lazily from the context.

use crate::{BehaviorNode, BoxNode, Status};

/// A boxed factory closure that builds a child subtree from the context.
type ProduceFn<D> = Box<dyn FnMut(&mut D) -> BoxNode<D>>;

/// Lazily produces a child subtree from the context, ticks it to completion,
/// then discards it.
///
/// On the first tick (or the first tick after the previous child finished) the
/// `produce` closure is called to build a fresh child from the current context.
/// That child is ticked until it succeeds or fails, then dropped so the next
/// run produces a new one. This is the idiomatic way to choose *what* to do
/// based on runtime state. Mirrors `ProducerNode`.
///
/// ```
/// use btree::prelude::*;
///
/// // Re-targets every time it (re)starts.
/// let mut p: Producer<i32> = Producer::new(|target: &mut i32| {
///     let goal = *target;
///     Action::new(move |cur: &mut i32| {
///         *cur += 1;
///         if *cur >= goal { Status::Success } else { Status::Running }
///     })
///     .boxed()
/// });
/// ```
pub struct Producer<D> {
    produce: ProduceFn<D>,
    label: Option<String>,
    running_node: Option<BoxNode<D>>,
}

impl<D> Producer<D> {
    /// Build an unlabeled producer from a factory closure.
    pub fn new(produce: impl FnMut(&mut D) -> BoxNode<D> + 'static) -> Self {
        Self {
            produce: Box::new(produce),
            label: None,
            running_node: None,
        }
    }

    /// Build a labeled producer (the label appears in traces).
    pub fn labeled(label: impl Into<String>, produce: impl FnMut(&mut D) -> BoxNode<D> + 'static) -> Self {
        Self {
            produce: Box::new(produce),
            label: Some(label.into()),
            running_node: None,
        }
    }
}

impl<D> BehaviorNode<D> for Producer<D> {
    fn tick(&mut self, data: &mut D) -> Status {
        if self.running_node.is_none() {
            let node = (self.produce)(data);
            self.running_node = Some(node);
        }
        // Unwrap is safe: we just ensured `running_node` is `Some`.
        let status = self.running_node.as_mut().unwrap().tick(data);
        match status {
            Status::Success => {
                self.running_node = None;
                Status::Success
            }
            Status::Failure => {
                self.running_node = None;
                Status::Failure
            }
            Status::Running => Status::Running,
        }
    }

    fn halt(&mut self) {
        if let Some(node) = self.running_node.as_mut() {
            node.halt();
        }
        self.running_node = None;
    }

    fn node_info(&self) -> String {
        let state = if self.running_node.is_some() {
            "Running"
        } else {
            "Halted"
        };
        match &self.label {
            Some(l) => format!("Producer : {l} : {state}"),
            None => format!("Producer : {state}"),
        }
    }
}
