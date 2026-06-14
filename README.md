# btree — an abstract behavior tree library for Rust

`btree` is a small, dependency-free [behavior tree](https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control))
toolkit. It is a Rust port of a Scala behavior-tree implementation and keeps the
same node vocabulary.

It is deliberately **engine-agnostic** — the whole library is generic over a
single *context* type and has no third-party dependencies, so the same trees run
in a unit test or inside a game engine such as Godot.

## Concepts

Every node implements one trait:

```rust
pub trait BehaviorNode<D> {
    fn tick(&mut self, data: &mut D) -> Status;     // required
    fn halt(&mut self) {}                            // cancel a running node
    fn node_info(&self) -> String { /* type name */ } // for debug traces
}
```

* **`D`** is the *tick data* / blackboard / agent, threaded by `&mut` through the
  whole tree on every tick.
* **`Status`** is `Success`, `Failure`, or `Running`. A `Running` node expects to
  be ticked again next frame.
* Composites store children as `BoxNode<D>` (`Box<dyn BehaviorNode<D>>`); the
  `nodes!` macro boxes a list for you.

## Node catalogue

| Node | Kind | Behavior |
|------|------|----------|
| `Predicate` | leaf | success/failure from a `FnMut(&mut D) -> bool` |
| `Action` | leaf | a `FnMut(&mut D) -> Status` (the main "do something" leaf) |
| `AlwaysSuccess` / `AlwaysFailure` / `AlwaysRunning` | leaf | constant result |
| `Sequence` | composite | run children in order, AND; keeps progress across ticks |
| `ReactiveSequence` | composite | like `Sequence` but restarts from the first child each tick |
| `ProgressiveSequence` | composite | "sequence star"; a failing child is retried, not reset |
| `FallbackSequence` | composite | run children in order, OR; keeps progress across ticks |
| `ReactiveFallbackSequence` | composite | like `FallbackSequence` but restarts each tick |
| `Invert` | decorator | swap success/failure |
| `ForceSuccess` / `ForceFailure` | decorator | coerce the finished result |
| `Repeat` | decorator | re-run a child up to N successes |
| `Retry` | decorator | re-run a failing child up to N times |
| `IfThenElse` | branch | pick a branch by predicate and latch onto it |
| `ReactiveIfThenElse` | branch | re-check the predicate every tick, switch branches |
| `Producer` | composite | lazily build a child subtree from the context, run it, discard it |

## Example

```rust
use btree::prelude::*;

struct Bot { energy: i32 }

let mut tree = ReactiveIfThenElse::labeled(
    "has_energy",
    |b: &mut Bot| b.energy > 0,
    Action::labeled("work", |b: &mut Bot| { b.energy -= 1; Status::Success }),
    Action::labeled("rest", |b: &mut Bot| { b.energy += 1; Status::Success }),
);

let mut bot = Bot { energy: 1 };
loop {
    if tree.tick(&mut bot).is_done() { break; }
}
```

## Debugging

Run a tree the normal way with `tick` — that path never allocates and has no
tracing overhead. When you need to understand *why* a tree did what it did, call
`tick_debug` instead. It does exactly the same work but also returns a
`DebugNode` tree of every node processed that tick and the `Status` it returned:

```rust
use btree::prelude::*;

let mut tree = Sequence::new(nodes![
    Predicate::labeled("ready", |_: &mut ()| true),
    Action::labeled("go", |_: &mut ()| Status::Running),
]);

let trace = tree.tick_debug(&mut ());
print!("{trace}");
// Sequence 1 / 2 [Running]
//   Predicate: ready : true [Success]
//   Action: go [Running]
```

Tracing is opt-in and pay-as-you-go, so you can leave `tick` in your hot loop
and only reach for `tick_debug` when inspecting behavior. `DebugNode` also
derives `Debug`/`Clone` and offers `.iter()` for a depth-first walk, so you can
assert on traces in tests or forward them to an in-engine inspector. Custom leaf
nodes get a sensible trace for free from the default `tick_debug`; custom
composites override `tick_debug` to record their children.

## Using it from Godot (gdext)

Make `D` your agent/blackboard type and tick the tree from `_process`:

```rust
// Pseudocode sketch — `Blackboard` is your own struct.
struct Enemy {
    tree: btree::BoxNode<Blackboard>,
    blackboard: Blackboard,
}

// in _process:
self.tree.tick(&mut self.blackboard);
```

The library never references Godot, so the `godot` crate only ever sees your own
context type.

## License

Licensed under either of MIT or Apache-2.0 at your option.
