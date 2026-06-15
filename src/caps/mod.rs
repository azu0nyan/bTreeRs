//! Capability traits a context type `D` can opt into.
//!
//! Some nodes need more from the context than the bare ability to be ticked —
//! a timer node needs to know how much time elapsed, a randomized node needs a
//! source of randomness. Rather than baking an engine clock or RNG into the
//! library (which would break its dependency-free, engine-agnostic promise),
//! those needs are expressed as small traits that the context implements:
//!
//! * [`HasDelta`] — the context can report the elapsed time of the current
//!   tick. Required by the timing nodes ([`Wait`](crate::Wait),
//!   [`Delay`](crate::Delay), [`Cooldown`](crate::Cooldown),
//!   [`TimeLimit`](crate::TimeLimit), [`RandomWait`](crate::RandomWait)).
//! * [`HasRng`] — the context can produce uniform random numbers. Required by
//!   the randomized nodes ([`RandomSequence`](crate::RandomSequence),
//!   [`RandomSelector`](crate::RandomSelector),
//!   [`ProbabilitySelector`](crate::ProbabilitySelector),
//!   [`Probability`](crate::Probability), [`RandomWait`](crate::RandomWait)).
//!
//! A node that needs a capability simply bounds the impl, e.g.
//! `impl<D: HasDelta> BehaviorNode<D> for Wait`, so it is only usable on
//! contexts that provide it. Nodes that need nothing extra stay generic over
//! any `D`.

mod has_delta;
mod has_rng;

pub use has_delta::HasDelta;
pub use has_rng::HasRng;
