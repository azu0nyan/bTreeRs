//! The [`Status`] returned by every behavior-node tick.

/// Result of ticking a [`BehaviorNode`](crate::BehaviorNode).
///
/// This mirrors the three-valued result used by virtually every behavior-tree
/// formulation:
///
/// * [`Status::Success`] — the node finished its work successfully.
/// * [`Status::Failure`] — the node finished, but failed.
/// * [`Status::Running`] — the node needs more ticks to finish; it should be
///   ticked again next frame.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Status {
    /// The node completed successfully.
    Success,
    /// The node completed, unsuccessfully.
    Failure,
    /// The node is still working and wants to be ticked again.
    Running,
}

impl Status {
    /// `true` for [`Status::Success`].
    #[inline]
    pub fn is_success(self) -> bool {
        matches!(self, Status::Success)
    }

    /// `true` for [`Status::Failure`].
    #[inline]
    pub fn is_failure(self) -> bool {
        matches!(self, Status::Failure)
    }

    /// `true` for [`Status::Running`].
    #[inline]
    pub fn is_running(self) -> bool {
        matches!(self, Status::Running)
    }

    /// `true` when the node has finished this tick, i.e. it is not
    /// [`Status::Running`].
    #[inline]
    pub fn is_done(self) -> bool {
        !self.is_running()
    }
}
