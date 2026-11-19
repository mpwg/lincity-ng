## Unreleased

###### User Interface
- Replaced the SDL desktop with the Godot 3D desktop. Classic save formats 2130
  and 2140 remain supported.

###### Internal
- Replaced the C++ simulation with RustCity and the C++ Godot binding with a
  Rust GDExtension. Cargo builds both Rust crates in one workspace.
- Archived the SDL desktop and C++ simulation on `archive/legacy-2d-cpp`.
