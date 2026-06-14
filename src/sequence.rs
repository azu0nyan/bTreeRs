//! Composite control nodes that run a list of children in order.
//!
//! Two families live here:
//!
//! * **Sequences** run children left-to-right and stop at the first failure
//!   (logical AND).
//! * **Fallbacks** (a.k.a. selectors) run children left-to-right and stop at
//!   the first success (logical OR).
//!
//! Each family comes in variants that differ in how they treat a `RUNNING` or
//! finished child between ticks:
//!
//! | Node                       | child FAILURE        | child RUNNING        |
//! |----------------------------|----------------------|----------------------|
//! | [`Sequence`]               | restart from start   | tick same child again |
//! | [`ReactiveSequence`]       | restart from start   | restart from start   |
//! | [`ProgressiveSequence`]    | tick same child again| tick same child again |
//!
//! (The fallbacks mirror this with the roles of success/failure swapped.)

use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// Halt every child node.
fn halt_all<D>(children: &mut [BoxNode<D>]) {
    for c in children.iter_mut() {
        c.halt();
    }
}

/// A sequence that remembers its progress between ticks ("sequence with
/// memory").
///
/// Ticks children in order. A child failure restarts the whole sequence and
/// returns [`Status::Failure`]; a running child is ticked again on the next
/// tick (progress is kept); when all children succeed the sequence resets and
/// returns [`Status::Success`]. Mirrors the Scala `Sequence`.
///
/// > Note: the Scala original advanced with `tasks.tail` (a bug that re-ran the
/// > second child forever); this port advances correctly by index, matching the
/// > documented "saves progress between ticks" intent.
pub struct Sequence<D> {
    children: Vec<BoxNode<D>>,
    current: usize,
}

impl<D> Sequence<D> {
    /// Build a sequence from its children. Use the [`nodes!`](crate::nodes)
    /// macro to box heterogeneous node types ergonomically.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self {
            children,
            current: 0,
        }
    }

    fn reset(&mut self) {
        self.current = 0;
        halt_all(&mut self.children);
    }
}

impl<D> BehaviorNode<D> for Sequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = loop {
            if self.current >= self.children.len() {
                self.reset();
                break Status::Success;
            }
            match tick_child(&mut *self.children[self.current], data, &mut dbg) {
                Status::Failure => {
                    self.reset();
                    break Status::Failure;
                }
                Status::Running => break Status::Running,
                Status::Success => self.current += 1,
            }
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        format!("Sequence {} / {}", self.current, self.children.len())
    }
}

/// A sequence that re-evaluates from the first child on every tick ("reactive
/// sequence").
///
/// Each tick starts over at the first child. The first running child causes the
/// node to return [`Status::Running`] (after halting any later children that
/// were left running from a previous tick); the first failure restarts and
/// fails. Mirrors `ReactiveSequence`.
pub struct ReactiveSequence<D> {
    children: Vec<BoxNode<D>>,
}

impl<D> ReactiveSequence<D> {
    /// Build a reactive sequence from its children.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self { children }
    }
}

impl<D> BehaviorNode<D> for ReactiveSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let len = self.children.len();
        let mut status = Status::Success;
        let mut completed = true;
        for i in 0..len {
            match tick_child(&mut *self.children[i], data, &mut dbg) {
                Status::Failure => {
                    halt_all(&mut self.children);
                    status = Status::Failure;
                    completed = false;
                    break;
                }
                Status::Running => {
                    // Cancel later children that may still be running from a
                    // previous tick.
                    for j in (i + 1)..len {
                        self.children[j].halt();
                    }
                    status = Status::Running;
                    completed = false;
                    break;
                }
                Status::Success => continue,
            }
        }
        if completed {
            halt_all(&mut self.children);
        }
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        halt_all(&mut self.children);
    }

    fn node_info(&self) -> String {
        format!("ReactiveSequence {}", self.children.len())
    }
}

/// A sequence that never re-runs a finished child until the whole sequence
/// completes (also known as `SequenceStar`).
///
/// Like [`Sequence`] it keeps progress across ticks, but a child failure does
/// *not* reset progress: the failing child is ticked again next tick. Mirrors
/// `ProgressiveSequence`.
pub struct ProgressiveSequence<D> {
    children: Vec<BoxNode<D>>,
    current: usize,
}

impl<D> ProgressiveSequence<D> {
    /// Build a progressive sequence from its children.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self {
            children,
            current: 0,
        }
    }

    fn reset(&mut self) {
        self.current = 0;
        halt_all(&mut self.children);
    }
}

impl<D> BehaviorNode<D> for ProgressiveSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = loop {
            if self.current >= self.children.len() {
                self.reset();
                break Status::Success;
            }
            match tick_child(&mut *self.children[self.current], data, &mut dbg) {
                // No reset: keep progress so the same child is retried.
                Status::Failure => break Status::Failure,
                Status::Running => break Status::Running,
                Status::Success => self.current += 1,
            }
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        format!("Progressive sequence {} / {}", self.current, self.children.len())
    }
}

/// A fallback / selector that remembers its progress between ticks.
///
/// Ticks children in order until one succeeds (then the node succeeds and
/// resets) or all fail (then the node fails). A running child returns
/// [`Status::Running`] and progress is kept. Mirrors `FallbackSequence`.
pub struct FallbackSequence<D> {
    children: Vec<BoxNode<D>>,
    current: usize,
}

impl<D> FallbackSequence<D> {
    /// Build a fallback (selector) from its children.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self {
            children,
            current: 0,
        }
    }

    fn reset(&mut self) {
        self.current = 0;
        halt_all(&mut self.children);
    }
}

impl<D> BehaviorNode<D> for FallbackSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let status = loop {
            if self.current >= self.children.len() {
                self.reset();
                break Status::Failure;
            }
            match tick_child(&mut *self.children[self.current], data, &mut dbg) {
                Status::Success => {
                    self.reset();
                    break Status::Success;
                }
                Status::Running => break Status::Running,
                Status::Failure => self.current += 1,
            }
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        format!("Fallback {} / {}", self.current, self.children.len())
    }
}

/// A fallback / selector that re-evaluates from the first child on every tick.
///
/// Each tick starts over at the first child; the first success wins, the first
/// running child returns [`Status::Running`] (after halting later children),
/// and a failing child is halted before moving on. Mirrors
/// `ReactiveFallbackSequence`.
pub struct ReactiveFallbackSequence<D> {
    children: Vec<BoxNode<D>>,
}

impl<D> ReactiveFallbackSequence<D> {
    /// Build a reactive fallback from its children.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self { children }
    }
}

impl<D> BehaviorNode<D> for ReactiveFallbackSequence<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let len = self.children.len();
        let mut status = Status::Failure;
        let mut completed = true;
        for i in 0..len {
            match tick_child(&mut *self.children[i], data, &mut dbg) {
                Status::Success => {
                    halt_all(&mut self.children);
                    status = Status::Success;
                    completed = false;
                    break;
                }
                Status::Running => {
                    for j in (i + 1)..len {
                        self.children[j].halt();
                    }
                    status = Status::Running;
                    completed = false;
                    break;
                }
                Status::Failure => self.children[i].halt(),
            }
        }
        if completed {
            halt_all(&mut self.children);
        }
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        halt_all(&mut self.children);
    }

    fn node_info(&self) -> String {
        format!("Reactive fallback {}", self.children.len())
    }
}
