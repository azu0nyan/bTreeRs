//! Shared helpers for the integration tests.
//!
//! Lives in a subdirectory (`tests/test_common/`) so cargo treats it as a
//! shared module rather than its own test binary. Each test file pulls it in
//! with `mod test_common;`.

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
