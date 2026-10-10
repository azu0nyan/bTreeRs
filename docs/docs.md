# btree — design notes

## Key design considerations

- Engine-agnostic by construction: generic over a single context type `D`,
  zero third-party dependencies. Any engine coupling (Godot, etc.) lives in a
  downstream crate (see `btree_godot`), never here.
- Small capability traits (`HasDelta`, `HasRng`, `HasNavAgent`, `FollowPath` in
  `src/caps`) let timing/randomness/navigation-dependent nodes stay
  deterministic and testable without pulling in a real clock/RNG/engine.
- Debug tracing is opt-in via `Option<&mut DebugNode>` threaded through `tick`
  so the hot path has zero allocation/formatting cost when tracing is off.

## Gotchas

- `Sequence`/`FallbackSequence` keep progress across ticks; their "Reactive"
  counterparts restart from the first child every tick — picking the wrong one
  is a common source of subtly wrong behavior trees.
- `RunCustomFunc` takes a bare `fn`, not a closure — use `Action` if you need
  to capture state.

## TODO

- Nothing tracked yet — add items here as they come up.
