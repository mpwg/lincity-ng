# Simulation reference contract

A case starts a fresh process with an input city, a traffic profile, a number of
days and an optional sequence of player commands. Relative day 0 is the loaded
state. On each subsequent day, commands execute in file order, followed by
animation at `1000 * day` milliseconds, one simulation tick and draining the
message queue. The city clock may already be nonzero when a case starts.

## Running a case

Build the standalone core with testing enabled. From the standalone build
directory, use the following commands, substituting the paths to the scenario
and comparison script. Use fresh output directories for each run.

```sh
tests/core_replay record decisions.txt /path/to/good_times.scn.gz no-traffic 1201 - reference
tests/core_replay replay decisions.txt /path/to/good_times.scn.gz no-traffic 1201 - candidate
python3 /path/to/lincity-core/tests/replay/compare.py reference candidate
```

Both `traffic` and `no-traffic` are supported. Replace `-` with an action file to
run commands. Each line contains the relative day, verb and x/y coordinates.
`build` takes an additional numeric construction group. The other verbs are
`query`, `evacuate`, `bulldoze` and `flood`. Blank lines and lines starting with
`#` are ignored. Recording and replay attach before loading the city.

## Decision tape v1

The header is `LINCITY-SIMULATION-TRACE 1`. Each subsequent line records one
event; integer fields use decimal notation. Reader and writer advance in
lockstep. The final `Z` must be followed by end of file.

| Record | Meaning |
| --- | --- |
| `P id x y group` | Placement. IDs start at one and follow placement order, including buildings loaded from disk. |
| `A milliseconds count ids…` | Animation visit order. |
| `D city_day count ids…` | Shuffled daily trade and update order. |
| `E city_day count ids…` | Visit order for a rocket evacuation pass. |
| `C value` | Integer returned by the C random source, before the caller's arithmetic. |
| `S site value` | Outcome consumed from a C++ distribution, identified by its call site. Examples include `vehicle.heading`, `rocket.success`, and `river.segment-length`. |

Replay validates placements, visit membership and sampled outcomes. A heading
must be available at its intersection. Consumers can apply a recorded choice
without reimplementing a C++ standard library distribution. Native recordings
still depend on the originating runtime, including the platform's `rand()`
range and evaluation order during map generation. The versioned small tape
does not require either random generator.

## Expected agreement

- `state.tsv` contains day, entity, field and value separated by tabs. The
  comparison matches fields by name within each day. Action and message indices
  preserve their sequence. Command observations include permission, rejection
  reason when denied, quote on visible tiles, execution result and cash change.
  Events include their type and payload, including coordinates where applicable.
- Each day includes the world clock, cash, technology, people pool and monthly
  population/housing accumulators. On day 0 all buildings and their ten stocks
  are emitted. Subsequent ticks report changes by map coordinate and commodity.
  A group of `-1` removes the building at that coordinate. An omitted stock
  retains its previous value.
- Compare all persisted fields at days 0, 1, 100 and the last day reached.
  Normalize XML indentation, attribute order and map entry order for this
  comparison. Preserve ordered histories and message order. The original
  six-scenario regression separately checks decompressed XML bytes and the
  legacy message stream against the pinned desktop implementation with the
  documented rail-crossing correction. Its XML comparison omits the new
  `diffusion-active` tile attribute, which the pinned writer did not emit.
  The save continuation case checks this attribute by comparing the next
  pollution tick. The build options describe comparison with the original
  crossing rule.
- The small smithy case has a versioned save, timed actions, a decision tape
  and a reviewed daily transcript. Its comparison test also changes a single
  goods value and checks for the first differing day, coordinate and field.
  The ten additional rules cases under `cases/v1` have their own plain-text
  inputs, expected outputs and fixed visit order.

## Running the checks

`ctest --output-on-failure` runs the reference cases. With Python 3 it also
checks the replay and its diagnostics. On Linux, if jemalloc is found, a
separate test replays `good_times` with that allocator in the child process.
Use the following command for the full 1,201-day allocator run, substituting
the local shared-library path. The default CTest run exercises three days.

```sh
python3 -B /path/to/lincity-core/tests/replay/check.py tests/core_replay scenario --days 1201 --allocator /path/to/libjemalloc.so.2
```

Keep the versioned inputs and expectations under review together. A correction
to game behaviour gets a targeted regression and an explanation of the intended
difference. The historical reference remains independently runnable using the
instructions in the repository's build options.
