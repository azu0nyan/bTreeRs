//! # btree — an abstract, engine-agnostic behavior tree library
//!
//! `btree` is a small, dependency-free behavior-tree toolkit. It is a Rust port
//! of a Scala behavior-tree implementation and keeps the same node vocabulary
//! (sequences, fallbacks, decorators, latching and reactive conditionals, a
//! lazy producer, predicate/action leaves).
//!
//! ## Design
//!
//! * Everything is built around one trait, [`BehaviorNode`], which is generic
//!   over a **context type `D`** — the "tick data" / blackboard / agent passed
//!   by `&mut` through the whole tree on each tick. The crate has **no engine
//!   dependency**, so the same trees run in tests or inside an engine such as
//!   Godot (give `D` your game/agent type and call [`BehaviorNode::tick`] from
//!   `_process`).
//! * Custom nodes are just types that implement [`BehaviorNode`]; only
//!   [`tick`](BehaviorNode::tick) is required.
//! * Composite children are stored as [`BoxNode<D>`](BoxNode) so heterogeneous
//!   node types compose freely. The [`nodes!`] macro boxes them for you.
//!
//! ## Example
//!
//! ```
//! use btree::prelude::*;
//!
//! struct Bot { energy: i32, ticks: i32 }
//!
//! // If we have energy, work; otherwise rest.
//! let mut tree = ReactiveIfThenElse::labeled(
//!     "has_energy",
//!     |b: &mut Bot| b.energy > 0,
//!     Action::labeled("work", |b: &mut Bot| { b.energy -= 1; b.ticks += 1; Status::Success }),
//!     Action::labeled("rest", |b: &mut Bot| { b.energy += 1; Status::Success }),
//! );
//!
//! let mut bot = Bot { energy: 1, ticks: 0 };
//! assert_eq!(tree.tick(&mut bot, None), Status::Success); // works, energy -> 0
//! assert_eq!(tree.tick(&mut bot, None), Status::Success); // rests, energy -> 1
//! assert_eq!(bot.ticks, 1);
//! ```
//!
//! ## Debugging
//!
//! Tracing is threaded through the single [`tick`](BehaviorNode::tick) method
//! via its `Option<&mut DebugNode>` argument. Pass `None` for the zero-overhead
//! production path; pass `Some(&mut slot)` (or use the
//! [`tick_traced`](BehaviorNode::tick_traced) convenience) to have each node
//! fill in a [`DebugNode`] tree of itself and the children it processed. The
//! tree `Display`s as an indented listing.
//!
//! ```
//! use btree::prelude::*;
//!
//! let mut tree = Sequence::new(nodes![
//!     Predicate::labeled("ready", |_: &mut ()| true),
//!     Action::labeled("go", |_: &mut ()| Status::Running),
//! ]);
//!
//! let (status, trace) = tree.tick_traced(&mut ());
//! assert_eq!(status, Status::Running);
//! assert_eq!(trace.children.len(), 2);
//! // `trace` renders as:
//! //   Sequence 1 / 2 [Running]
//! //     Predicate: ready : true [Success]
//! //     Action: go [Running]
//! ```
//!
//! See the [`prelude`] for the full set of node types.

#![warn(missing_docs)]

mod branch;
mod debug;
mod decorator;
mod leaf;
mod node;
mod producer;
mod sequence;
mod status;

pub use branch::{IfThenElse, ReactiveIfThenElse};
pub use debug::{record, tick_child, DebugNode};
pub use decorator::{ForceFailure, ForceSuccess, Invert, Repeat, Retry};
pub use leaf::{Action, AlwaysFailure, AlwaysRunning, AlwaysSuccess, Predicate};
pub use node::{BehaviorNode, BoxNode, IntoBoxNode};
pub use producer::Producer;
pub use sequence::{
    FallbackSequence, ProgressiveSequence, ReactiveFallbackSequence, ReactiveSequence, Sequence,
};
pub use status::Status;

/// Build a `Vec<BoxNode<D>>` from a comma-separated list of nodes, boxing each.
///
/// This is the ergonomic way to pass children to the composite constructors,
/// which take a `Vec<BoxNode<D>>`.
///
/// ```
/// use btree::prelude::*;
///
/// let mut seq: Sequence<()> = Sequence::new(nodes![
///     AlwaysSuccess,
///     AlwaysSuccess,
/// ]);
/// assert_eq!(seq.tick(&mut (), None), Status::Success);
/// ```
#[macro_export]
macro_rules! nodes {
    ($($node:expr),* $(,)?) => {
        ::std::vec![$($crate::IntoBoxNode::boxed($node)),*]
    };
}

/// Re-exports of everything you typically need to build and tick a tree.
pub mod prelude {
    pub use crate::debug::{record, tick_child, DebugNode};
    pub use crate::node::{BehaviorNode, BoxNode, IntoBoxNode};
    pub use crate::status::Status;

    pub use crate::branch::{IfThenElse, ReactiveIfThenElse};
    pub use crate::decorator::{ForceFailure, ForceSuccess, Invert, Repeat, Retry};
    pub use crate::leaf::{Action, AlwaysFailure, AlwaysRunning, AlwaysSuccess, Predicate};
    pub use crate::producer::Producer;
    pub use crate::sequence::{
        FallbackSequence, ProgressiveSequence, ReactiveFallbackSequence, ReactiveSequence,
        Sequence,
    };

    pub use crate::nodes;
}
