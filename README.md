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
    // required; `dbg` is an optional trace slot (pass None for the fast path)
    fn tick(&mut self, data: &mut D, dbg: Option<&mut DebugNode>) -> Status;
    fn halt(&mut self) {}                             // cancel a running node
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
| `RunCustomFunc` | leaf | a bare `fn(&mut D) -> Status` — the allocation-free `Action` sibling |
| `AlwaysSuccess` / `AlwaysFailure` / `AlwaysRunning` | leaf | constant result |
| `WaitTicks` | leaf | run for N ticks, then succeed |
| `Wait` | leaf † | run for N seconds, then succeed |
| `RandomWait` | leaf †‡ | run for a random duration in `[min, max]` seconds |
| `SetDestination` | leaf § | ask the nav agent to path to its target; forwards the pathfinding status |
| `GoTo` | leaf §† | move the agent along its path each tick until it arrives |
| `Sequence` | composite | run children in order, AND; keeps progress across ticks |
| `ReactiveSequence` | composite | like `Sequence` but restarts from the first child each tick |
| `ProgressiveSequence` | composite | "sequence star"; a failing child is retried, not reset |
| `FallbackSequence` | composite | run children in order, OR; keeps progress across ticks |
| `ReactiveFallbackSequence` | composite | like `FallbackSequence` but restarts each tick |
| `Parallel` | composite | tick all children each tick; resolve by success/failure policies |
| `ForEach` | composite | run one child once per item of a context-supplied collection |
| `RandomSequence` / `RandomSelector` | composite ‡ | `Sequence` / selector run in a shuffled order each run |
| `ProbabilitySelector` | composite ‡ | pick one child by weighted chance and run it |
| `Invert` | decorator | swap success/failure |
| `ForceSuccess` / `ForceFailure` | decorator | coerce the finished result |
| `Repeat` | decorator | re-run a child up to N successes |
| `Retry` | decorator | re-run a failing child up to N times |
| `RepeatUntilSuccess` / `RepeatUntilFailure` | decorator | re-run a child until it succeeds / fails |
| `RunLimit` | decorator | let a child finish at most N times, then fail |
| `Delay` | decorator † | wait N seconds, then run the child |
| `Cooldown` | decorator † | run a child no more often than once per N seconds |
| `TimeLimit` | decorator † | fail a child that runs longer than N seconds |
| `Probability` | decorator ‡ | run a child only with probability `p`, else fail |
| `IfThenElse` | branch | pick a branch by predicate and latch onto it |
| `ReactiveIfThenElse` | branch | re-check the predicate every tick, switch branches |
| `Producer` | composite | lazily build a child subtree from the context, run it, discard it |

**† needs `D: HasDelta`** — the context reports the current tick's elapsed time
(`fn delta_seconds(&self) -> f64`); stash your frame `delta` into the context
before ticking. **‡ needs `D: HasRng`** — the context supplies uniform random
draws (`fn next_f64(&mut self) -> f64`). **§ needs a navigation capability** —
`SetDestination` needs `D: HasNavAgent` (`fn set_destination(&mut self) -> Status`,
returning `Failure` if no path, `Running` while computing, `Success` when one is
ready); `GoTo` needs `D: FollowPath` (`fn advance(&mut self, delta: f64) -> Status`)
and `D: HasDelta`. These small capability traits live in
[`btree::caps`](src/caps) and keep the timing / randomized / navigation nodes
engine-free and deterministic in tests, while still working against a real engine
clock, RNG, and navigation agent.

Each node type lives in its own file under the [`nodes`](src/nodes) module
(`btree::nodes::Sequence`, …) and is re-exported at the crate root and from the
prelude, so `btree::Sequence` and `btree::nodes::Sequence` are the same type.
The core trait, `Status`, the capability traits, and the debug machinery stay at
the top level (`node.rs`, `status.rs`, `caps/`, `debug.rs`).

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
    if tree.tick(&mut bot, None).is_done() { break; }
}
```

## Debugging

Tracing rides along on the single `tick` method through its
`Option<&mut DebugNode>` argument:

* Pass `None` for the normal path — nothing is recorded, no allocation, no
  formatting. Leave this in your hot loop.
* Pass `Some(&mut slot)` and every node fills in its name, status, and the
  slots of the children it processed, building a tree. The `tick_traced`
  convenience does this for you and hands back the finished `DebugNode`:

```rust
use btree::prelude::*;

let mut tree = Sequence::new(nodes![
    Predicate::labeled("ready", |_: &mut ()| true),
    Action::labeled("go", |_: &mut ()| Status::Running),
]);

let (status, trace) = tree.tick_traced(&mut ());
print!("{trace}");
// Sequence 1 / 2 [Running]
//   Predicate: ready : true [Success]
//   Action: go [Running]
```

`DebugNode` derives `Debug`/`Clone` and offers `.iter()` for a depth-first
walk, so you can assert on traces in tests or forward them to an in-engine
inspector. When writing your own nodes, fill the slot with the `record` helper
(leaves) and `tick_child` helper (composites) — both no-op when the slot is
`None`, so tracing stays free unless you ask for it.

## Using it from Godot (gdext)

Make `D` your agent/blackboard type and tick the tree from `_process`:

```rust
// Pseudocode sketch — `Blackboard` is your own struct.
struct Enemy {
    tree: btree::BoxNode<Blackboard>,
    blackboard: Blackboard,
}

// in _process:
self.tree.tick(&mut self.blackboard, None);
```

The library never references Godot, so the `godot` crate only ever sees your own
context type.

## License

Licensed under either of MIT or Apache-2.0 at your option.
