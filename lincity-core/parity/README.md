# C++ reference cases

The headless C++ runner reads `cases/v1/input.txt` and checks its transcript against
`cases/v1/expected.txt`, recorded using LinCity::Core. Each input line starts
a fresh case. The output echoes the case before its results.

| Case and input fields | Output per observation, in order |
| --- | --- |
| `names` | Each commodity's canonical save name and parsed numeric ID in enum order, followed by the canonical result for each invalid spelling (`Food`, empty string, `unknown`). |
| `inventory amount capacity add_amount add_capacity subtract_amount subtract_capacity` | Amount, capacity and fill in thousandths, initially and after each operation. |
| `stats population housed food capacity goods_tax transport births unemployment_history boundaries` | After each boundary (`d` daily, `m` monthly, `y` yearly): population accumulator and finalized value, population history length, newest and previous population, current unhoused history, food amount and capacity accumulators, food history length, newest food amount and capacity, previous food fill in thousandths, accumulator and finalized value for each of goods tax, transport expenses and births, then the IEEE 754 binary32 bits of unemployment history as an unsigned decimal integer. |
| `chain labor solar_high station_high station_low ticks remove_at` | Initial state and every tick: tick, solar labor, solar high voltage, station high and low voltage (or `removed`). Remove the station before the indicated tick. Zero keeps it throughout the case. |

The chain places an adjacent solar plant and substation at maximum technology.
Each tick visits the solar plant and then the substation, performing trade before
each building's update as in the simulation. Fixing the visit order isolates the
trade and production rules from the world's shuffle. The fixtures cover rate-limited
flow, integer rounding during labor use and power exchange, refused conversion when
low-voltage storage is full, and several ticks after bulldozing a stockholding station.
The statistics slice covers the fields listed above. The case files use plain text and can be consumed independently of CMake and C++.

Build `core_parity_reference` and run `ctest --test-dir <build-dir> -R core.parity --output-on-failure`.
For an intentional rule change, review the new C++ transcript against the previous
oracle before updating the expected file.
