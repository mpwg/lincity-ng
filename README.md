# LinCity-NG: A city simulation game

https://github.com/mpwg/lincity-ng

## Introduction

LinCity-NG is a city simulation game. It is a polished and improved
version of the classic LinCity game. In the game, you are required
to build and maintain a city. You can win the game either by
building a sustainable economy or by evacuating all citizens with
spaceships.

See the in-game help [F1] for how to play.

## Build and run

The Godot 4.5.1 desktop uses RustCity for the simulation, city generation, and
saved games. Its main menu can start a new city, choose a scenario, or continue
a saved city. [Dependencies](docs/DEPENDENCIES.md) · [Build options](docs/BUILD_OPTIONS.md)

Run these commands from the repository root.

```sh
cmake -S . -B build/godot-editor -DCMAKE_BUILD_TYPE=Release
cmake --build build/godot-editor --target lincity_godot --parallel
godot --headless --path godot --import
godot --path godot
```

Run the Rust tests with `cargo test --locked --manifest-path rustcity-core/Cargo.toml`.
[Desktop controls, tests, and release exports](docs/GODOT_DESKTOP.md)

## Saved games and the classic desktop

Godot opens classic LinCity-NG saves in formats 2130 and 2140 through the load
dialog. For saves from LinCity-NG 2.12.x, follow the conversion instructions in
the archived README.
[Save conversion instructions](https://github.com/mpwg/lincity-ng/blob/archive/legacy-2d-cpp/README.md#loading-games-from-versions-prior-to-2130)

The archive branch preserves the SDL desktop, the original C++ simulation, and
the instructions and reports for comparing it with RustCity.
[Source archive](https://github.com/mpwg/lincity-ng/tree/archive/legacy-2d-cpp) · [Parity results](https://github.com/mpwg/lincity-ng/blob/archive/legacy-2d-cpp/docs/rustcity-core-paritaet.md)

## Contributing

[Contributing guidelines](CONTRIBUTING.md) · [Issue tracker](https://github.com/mpwg/lincity-ng/issues)

## License

Copyright (C) 1995-2026 LinCity-NG contributors

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License along
with this program; if not, write to the Free Software Foundation, Inc.,
51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
