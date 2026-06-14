//! The core [`BehaviorNode`] trait and node-boxing helpers.

use crate::{DebugNode, Status};

/// A boxed, type-erased behavior node operating over the context type `D`.
///
/// Composite nodes (sequences, decorators, …) store their children as
/// `BoxNode<D>` so that heterogeneous node types can live in the same tree.
pub type BoxNode<D> = Box<dyn BehaviorNode<D>>;

/// A node in a behavior tree.
///
/// `D` is the *tick data* / context type threaded through the whole tree on
/// every tick. It is passed by `&mut` so leaf nodes can both read and mutate
/// the agent / blackboard they act on. The library is fully generic over `D`
/// and never depends on any engine, so the same tree definition works on a
/// plain struct in a unit test or on a Godot node in a game.
///
/// Implement this trait to add your own custom leaves or composites. The only
/// required method is [`tick`](BehaviorNode::tick); [`halt`](BehaviorNode::halt)
/// and [`node_info`](BehaviorNode::node_info) have sensible defaults.
///
/// ## Tracing
///
/// `tick` takes an optional debug slot, `Option<&mut DebugNode>`. Pass `None`
/// for the normal zero-overhead path; pass `Some(&mut slot)` to have the node
/// record a [`DebugNode`] trace of itself and the children it processes. Custom
/// implementations fill the slot with the [`record`](crate::record) helper
/// (leaves) and [`tick_child`](crate::tick_child) helper (composites), both of
/// which no-op when the slot is `None`.
pub trait BehaviorNode<D> {
    /// Advance this node by one tick, returning its [`Status`].
    ///
    /// A node that returns [`Status::Running`] expects to be ticked again on a
    /// later frame and should preserve whatever state it needs between calls.
    ///
    /// `dbg` is an optional trace slot. When it is `Some`, the node must fill it
    /// with its own name and status (via [`record`](crate::record)) and, for a
    /// composite, the traces of every child it ticks (via
    /// [`tick_child`](crate::tick_child)). When it is `None`, no tracing work is
    /// done.
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status;

    /// Abort a node that is currently [`Status::Running`].
    ///
    /// Composite nodes propagate `halt` to their running children so that a
    /// reactive parent can cleanly cancel a branch it is no longer interested
    /// in. The default implementation does nothing, which is correct for
    /// stateless leaves.
    fn halt(&mut self) {}

    /// A short human-readable description of this node, used when rendering a
    /// debug trace. The default returns the node's (unqualified) type name;
    /// stateful nodes override it to surface their progress.
    fn node_info(&self) -> String {
        short_type_name::<Self>()
    }

    /// Tick this node with tracing enabled, returning both the [`Status`] and a
    /// fully populated [`DebugNode`] trace.
    ///
    /// This is just a convenience wrapper around `tick(data, Some(..))`; the
    /// plain [`tick`](BehaviorNode::tick) with `None` remains the zero-overhead
    /// path for production use.
    fn tick_traced(&mut self, data: &mut D) -> (Status, DebugNode) {
        let mut root = DebugNode::empty();
        let status = self.tick(data, Some(&mut root));
        (status, root)
    }
}

/// Returns the unqualified type name of `T`, e.g. `Sequence` rather than
/// `btree::sequence::Sequence<my_game::Ctx>`.
pub(crate) fn short_type_name<T: ?Sized>() -> String {
    let full = std::any::type_name::<T>();
    let before_generics = full.split('<').next().unwrap_or(full);
    before_generics
        .rsplit("::")
        .next()
        .unwrap_or(before_generics)
        .to_string()
}

/// Forwarding impl so a `Box<dyn BehaviorNode<D>>` (or any boxed node) is itself
/// a [`BehaviorNode`]. This lets boxed nodes be used anywhere an
/// `impl BehaviorNode<D>` is expected.
impl<D, N: BehaviorNode<D> + ?Sized> BehaviorNode<D> for Box<N> {
    #[inline]
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status {
        (**self).tick(data, dbg)
    }

    #[inline]
    fn halt(&mut self) {
        (**self).halt()
    }

    #[inline]
    fn node_info(&self) -> String {
        (**self).node_info()
    }

    #[inline]
    fn tick_traced(&mut self, data: &mut D) -> (Status, DebugNode) {
        (**self).tick_traced(data)
    }
}

/// Convenience for turning any concrete node into a [`BoxNode`].
///
/// ```
/// use btree::prelude::*;
///
/// let node: BoxNode<()> = AlwaysSuccess.boxed();
/// ```
pub trait IntoBoxNode<D> {
    /// Box this node up as a type-erased [`BoxNode`].
    fn boxed(self) -> BoxNode<D>;
}

impl<D, T: BehaviorNode<D> + 'static> IntoBoxNode<D> for T {
    #[inline]
    fn boxed(self) -> BoxNode<D> {
        Box::new(self)
    }
}
