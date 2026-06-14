//! Concrete behavior-tree nodes — one type per submodule.
//!
//! These are the building blocks you compose into a tree:
//!
//! * **Leaves** that look at and act on the context: [`Predicate`], [`Action`],
//!   [`AlwaysSuccess`], [`AlwaysFailure`], [`AlwaysRunning`].
//! * **Decorators** that transform a single child: [`Invert`], [`ForceSuccess`],
//!   [`ForceFailure`], [`Repeat`], [`Retry`].
//! * **Composites** that run a list of children: the sequence family
//!   ([`Sequence`], [`ReactiveSequence`], [`ProgressiveSequence`]) and the
//!   fallback / selector family ([`FallbackSequence`],
//!   [`ReactiveFallbackSequence`]).
//! * **Conditional branches**: [`IfThenElse`] (latching) and
//!   [`ReactiveIfThenElse`] (re-evaluating).
//! * The lazy [`Producer`].
//!
//! Every type here implements [`BehaviorNode`] over a context type `D`. They
//! are re-exported at the crate root (and from the [`prelude`](crate::prelude)),
//! so `btree::Sequence` and
//! `btree::nodes::Sequence` name the same type.

use crate::{BehaviorNode, BoxNode};

// Leaves
mod action;
mod always_failure;
mod always_running;
mod always_success;
mod predicate;

// Decorators
mod force_failure;
mod force_success;
mod invert;
mod repeat;
mod retry;

// Sequences & fallbacks
mod fallback_sequence;
mod progressive_sequence;
mod reactive_fallback_sequence;
mod reactive_sequence;
mod sequence;

// Conditional branches
mod if_then_else;
mod reactive_if_then_else;

// Producer
mod producer;

pub use action::Action;
pub use always_failure::AlwaysFailure;
pub use always_running::AlwaysRunning;
pub use always_success::AlwaysSuccess;
pub use predicate::Predicate;

pub use force_failure::ForceFailure;
pub use force_success::ForceSuccess;
pub use invert::Invert;
pub use repeat::Repeat;
pub use retry::Retry;

pub use fallback_sequence::FallbackSequence;
pub use progressive_sequence::ProgressiveSequence;
pub use reactive_fallback_sequence::ReactiveFallbackSequence;
pub use reactive_sequence::ReactiveSequence;
pub use sequence::Sequence;

pub use if_then_else::IfThenElse;
pub use reactive_if_then_else::ReactiveIfThenElse;

pub use producer::Producer;

/// Halt every child node. Shared by the sequence / fallback composites.
fn halt_all<D>(children: &mut [BoxNode<D>]) {
    for c in children.iter_mut() {
        c.halt();
    }
}

/// Which branch of a conditional is currently running. Shared by
/// [`IfThenElse`] and [`ReactiveIfThenElse`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Branch {
    Then,
    Else,
}

/// Render a (possibly absent) running branch for `node_info`.
fn branch_word(running: Option<Branch>) -> &'static str {
    match running {
        Some(Branch::Then) => "then",
        Some(Branch::Else) => "else",
        None => "NO BRANCH",
    }
}
