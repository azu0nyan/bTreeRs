//! Concrete behavior-tree nodes — one type per submodule.
//!
//! These are the building blocks you compose into a tree:
//!
//! * **Leaves** that look at and act on the context: [`Predicate`], [`Action`],
//!   [`RunCustomFunc`], [`AlwaysSuccess`], [`AlwaysFailure`], [`AlwaysRunning`],
//!   the timers [`Wait`], [`WaitTicks`] and [`RandomWait`], and the navigation
//!   leaves [`SetDestination`] and [`GoTo`].
//! * **Decorators** that transform a single child: [`Invert`], [`ForceSuccess`],
//!   [`ForceFailure`], [`Repeat`], [`Retry`], [`RepeatUntilSuccess`],
//!   [`RepeatUntilFailure`], [`RunLimit`], [`Delay`], [`Cooldown`],
//!   [`TimeLimit`], [`Probability`].
//! * **Composites** that run a list of children: the sequence family
//!   ([`Sequence`], [`ReactiveSequence`], [`ProgressiveSequence`]), the fallback
//!   / selector family ([`FallbackSequence`], [`ReactiveFallbackSequence`]),
//!   [`Parallel`], [`ForEach`], and the randomized
//!   [`RandomSequence`] / [`RandomSelector`] / [`ProbabilitySelector`].
//! * **Conditional branches**: [`IfThenElse`] (latching) and
//!   [`ReactiveIfThenElse`] (re-evaluating).
//! * The lazy [`Producer`].
//!
//! Every type here implements [`BehaviorNode`] over a context type `D`. Some
//! bound `D` on a capability trait (see [`caps`](crate::caps)): the timers need
//! [`HasDelta`](crate::HasDelta), the randomized nodes need [`HasRng`], and the
//! navigation leaves need [`HasNavAgent`](crate::HasNavAgent) /
//! [`FollowPath`](crate::FollowPath). They are re-exported at the crate root (and from
//! the [`prelude`](crate::prelude)), so `btree::Sequence` and
//! `btree::nodes::Sequence` name the same type.

use crate::caps::HasRng;
use crate::{BehaviorNode, BoxNode};

// Leaves
mod action;
mod always_failure;
mod always_running;
mod always_success;
mod go_to;
mod predicate;
mod random_wait;
mod run_custom_func;
mod set_destination;
mod wait;
mod wait_ticks;

// Decorators
mod cooldown;
mod delay;
mod force_failure;
mod force_success;
mod invert;
mod probability;
mod repeat;
mod repeat_until_failure;
mod repeat_until_success;
mod retry;
mod run_limit;
mod time_limit;

// Sequences, fallbacks & other composites
mod fallback_sequence;
mod for_each;
mod parallel;
mod probability_selector;
mod progressive_sequence;
mod random_selector;
mod random_sequence;
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
pub use go_to::GoTo;
pub use predicate::Predicate;
pub use random_wait::RandomWait;
pub use run_custom_func::RunCustomFunc;
pub use set_destination::SetDestination;
pub use wait::Wait;
pub use wait_ticks::WaitTicks;

pub use cooldown::Cooldown;
pub use delay::Delay;
pub use force_failure::ForceFailure;
pub use force_success::ForceSuccess;
pub use invert::Invert;
pub use probability::Probability;
pub use repeat::Repeat;
pub use repeat_until_failure::RepeatUntilFailure;
pub use repeat_until_success::RepeatUntilSuccess;
pub use retry::Retry;
pub use run_limit::RunLimit;
pub use time_limit::TimeLimit;

pub use fallback_sequence::FallbackSequence;
pub use for_each::ForEach;
pub use parallel::{Parallel, ParallelPolicy};
pub use probability_selector::ProbabilitySelector;
pub use progressive_sequence::ProgressiveSequence;
pub use random_selector::RandomSelector;
pub use random_sequence::RandomSequence;
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

/// A shuffled `0..len` index order drawn from the context RNG. Shared by the
/// randomized composites ([`RandomSequence`] / [`RandomSelector`]).
fn shuffled_order<D: HasRng>(len: usize, data: &mut D) -> Vec<usize> {
    let mut order: Vec<usize> = (0..len).collect();
    // Fisher–Yates.
    for i in (1..len).rev() {
        let j = data.next_below(i + 1);
        order.swap(i, j);
    }
    order
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
