//! The [`Parallel`] composite and its [`ParallelPolicy`].

use super::halt_all;
use crate::debug::{record, tick_child};
use crate::{BehaviorNode, BoxNode, DebugNode, Status};

/// When a [`Parallel`] composite considers a success or failure condition met.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParallelPolicy {
    /// The condition is met only once *every* child has reached it.
    RequireAll,
    /// The condition is met as soon as *one* child reaches it.
    RequireOne,
}

/// Ticks all of its still-running children every tick, resolving by a pair of
/// [`ParallelPolicy`] rules. Mirrors `BTParallel`.
///
/// Each tick, every child that has not yet finished is ticked once. A child
/// that returns [`Status::Success`] or [`Status::Failure`] is recorded as done
/// and is not ticked again until the composite resets. The composite then
/// resolves:
///
/// * **Failure** if the `failure` policy is met (`RequireOne`: at least one
///   child failed; `RequireAll`: all children failed), else
/// * **Success** if the `success` policy is met (`RequireOne`: at least one
///   child succeeded; `RequireAll`: all children succeeded), else
/// * [`Status::Running`].
///
/// On resolving it halts every child and clears its bookkeeping, so the next
/// entry runs all children afresh. [`Parallel::new`] uses the common "all must
/// succeed, any failure aborts" configuration.
pub struct Parallel<D> {
    children: Vec<BoxNode<D>>,
    results: Vec<Option<Status>>,
    success: ParallelPolicy,
    failure: ParallelPolicy,
}

impl<D> Parallel<D> {
    /// Build a parallel that succeeds when all children succeed and fails as
    /// soon as one child fails.
    pub fn new(children: Vec<BoxNode<D>>) -> Self {
        Self::with_policies(
            children,
            ParallelPolicy::RequireAll,
            ParallelPolicy::RequireOne,
        )
    }

    /// Build a parallel with explicit `success` and `failure` policies.
    pub fn with_policies(
        children: Vec<BoxNode<D>>,
        success: ParallelPolicy,
        failure: ParallelPolicy,
    ) -> Self {
        let results = vec![None; children.len()];
        Self {
            children,
            results,
            success,
            failure,
        }
    }

    fn reset(&mut self) {
        halt_all(&mut self.children);
        for r in self.results.iter_mut() {
            *r = None;
        }
    }
}

impl<D> BehaviorNode<D> for Parallel<D> {
    fn tick(&mut self, data: &mut D, mut dbg: Option<&mut DebugNode>) -> Status {
        let len = self.children.len();
        for i in 0..len {
            if self.results[i].is_none() {
                let s = tick_child(&mut *self.children[i], data, &mut dbg);
                if s.is_done() {
                    self.results[i] = Some(s);
                }
            }
        }

        let successes = self
            .results
            .iter()
            .filter(|r| **r == Some(Status::Success))
            .count();
        let failures = self
            .results
            .iter()
            .filter(|r| **r == Some(Status::Failure))
            .count();

        let failed = match self.failure {
            ParallelPolicy::RequireOne => failures >= 1,
            ParallelPolicy::RequireAll => failures == len,
        };
        let succeeded = match self.success {
            ParallelPolicy::RequireOne => successes >= 1,
            ParallelPolicy::RequireAll => successes == len,
        };

        let status = if failed {
            self.reset();
            Status::Failure
        } else if succeeded {
            self.reset();
            Status::Success
        } else {
            Status::Running
        };
        record(dbg, status, || self.node_info());
        status
    }

    fn halt(&mut self) {
        self.reset();
    }

    fn node_info(&self) -> String {
        let done = self.results.iter().filter(|r| r.is_some()).count();
        format!("Parallel {} / {}", done, self.children.len())
    }
}
