# LinCity-NG: A city simulation game

https://github.com/mpwg/lincity-ng

LinCity-NG is a 3D city simulation based on the classic LinCity. Build a city
and win by sustaining its economy or evacuating every resident by rocket. Press
F1 for the in-game help.

## Build and run

Install Godot 4.5.1, Rust and Cargo 1.94 or newer, and your platform's system
linker. Cargo builds the RustCity simulation and the Rust GDExtension that
connects it to the Godot desktop. [Dependencies](docs/DEPENDENCIES.md)

Run these commands from the repository root.

```sh
cargo godot-build
godot --headless --path godot --import
godot --path godot
```

`cargo godot-build` stages the native library in `godot/bin/`. Use
`cargo godot-build --export` to stage the library for a release export, and
`cargo test --workspace --locked` to run the Rust tests.
[Build options](docs/BUILD_OPTIONS.md) · [Desktop controls, tests, and exports](docs/GODOT_DESKTOP.md)

## Project layout

- `godot/` contains the desktop interface, scenarios, and game assets.
- `godot/native/` contains the Rust GDExtension and the `CityWorld` API used by GDScript.
- `rustcity-core/` contains the simulation and save-game handling.

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
