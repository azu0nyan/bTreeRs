//! Shared helpers for the integration tests.
//!
//! Lives in a subdirectory (`tests/common/`) so cargo treats it as a
//! shared module rather than its own test binary. Each test file pulls it in
//! with `mod common;`.

#![allow(dead_code)]

use btree::prelude::*;

/// A tiny context that records the order in which actions ran.
#[derive(Default)]
pub struct Ctx {
    pub log: Vec<&'static str>,
}

/// An action labeled `tag` that records the tag in the context log and returns
/// a fixed status.
pub fn tagged(tag: &'static str, status: Status) -> Action<Ctx> {
    Action::labeled(tag, move |c: &mut Ctx| {
        c.log.push(tag);
        status
    })
}

/// A context for the timing / randomized nodes: a fixed per-tick delta, a log
/// of which actions ran, and a deterministic RNG that replays a fixed queue of
/// draws (cycling if exhausted).
pub struct Sim {
    pub delta: f64,
    pub log: Vec<&'static str>,
    rng: Vec<f64>,
    rng_at: usize,
}

impl Sim {
    /// A context whose every tick reports `delta` seconds and whose RNG always
    /// draws `0.0` (i.e. picks the lowest / first option).
    pub fn new(delta: f64) -> Self {
        Self::with_rng(delta, vec![0.0])
    }

    /// A context with a fixed `delta` and a queue of RNG draws to replay.
    pub fn with_rng(delta: f64, rng: Vec<f64>) -> Self {
        Self {
            delta,
            log: Vec::new(),
            rng,
            rng_at: 0,
        }
    }
}

/// An action labeled `tag` that records the tag in a [`Sim`] log and returns a
/// fixed status.
pub fn sim_tagged(tag: &'static str, status: Status) -> Action<Sim> {
    Action::labeled(tag, move |c: &mut Sim| {
        c.log.push(tag);
        status
    })
}

impl HasDelta for Sim {
    fn delta_seconds(&self) -> f64 {
        self.delta
    }
}

impl HasRng for Sim {
    fn next_f64(&mut self) -> f64 {
        let v = self.rng[self.rng_at % self.rng.len()];
        self.rng_at += 1;
        v
    }
}
