# LinCity Core

This directory contains the city simulation, including its building modules,
save/load implementation, animation metadata and test scenarios. The source
directory can be copied and built on its own.

The build requires CMake 3.21 or later, a C++17 compiler, zlib, libxml2,
libxml++ 5.0, fmt 9 or later and pkg-config. Intl is optional. The standalone
build uses no SDL, desktop assets or desktop build tooling.

From this directory, configure, build and run the simulation tests.

```sh
cmake -S . -B build -DBUILD_TESTING=ON
cmake --build build
ctest --test-dir build --output-on-failure
```

A project can add this directory with `add_subdirectory(lincity-core)` and
link its own target to `LinCity::Core`. The simulation headers are available
under `lincity/`, including `lincity/world.hpp` and `lincity/UserOperation.hpp`.

The [simulation reference contract](parity/PARITY-CONTRACT.md) describes
deterministic scenario replay and field-level comparisons. The
[versioned rules cases](parity/README.md) cover inventories, accounting
periods and a production and trade chain using the C++ core.
