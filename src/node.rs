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
pub trait BehaviorNode<D> {
    /// Advance this node by one tick, returning its [`Status`].
    ///
    /// A node that returns [`Status::Running`] expects to be ticked again on a
    /// later frame and should preserve whatever state it needs between calls.
    fn tick(&mut self, data: &mut D) -> Status;

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

    /// Tick this node **and** record a [`DebugNode`] trace of everything that
    /// was processed.
    ///
    /// This does the same work as [`tick`](BehaviorNode::tick) but additionally
    /// returns a tree of the processed node names and their statuses, which is
    /// invaluable for debugging why a tree behaved the way it did. It is fully
    /// optional: the plain [`tick`](BehaviorNode::tick) path never builds a
    /// trace, so tracing costs nothing unless you ask for it.
    ///
    /// The default implementation is correct for leaf nodes (it ticks and
    /// returns a childless trace). Composite nodes override it to also trace
    /// the children they processed.
    fn tick_debug(&mut self, data: &mut D) -> DebugNode {
        let status = self.tick(data);
        DebugNode::leaf(self.node_info(), status)
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
    fn tick(&mut self, data: &mut D) -> Status {
        (**self).tick(data)
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
    fn tick_debug(&mut self, data: &mut D) -> DebugNode {
        (**self).tick_debug(data)
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
