find_program(RUSTCITY_CARGO cargo REQUIRED)
set(manifest "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml")
set(cargo_target "${CMAKE_CURRENT_BINARY_DIR}/cargo")
set(cargo_platform_dir "${cargo_target}")
if(CMAKE_CROSSCOMPILING AND APPLE AND CMAKE_OSX_ARCHITECTURES STREQUAL "arm64")
  set(cargo_platform aarch64-apple-darwin)
elseif(CMAKE_CROSSCOMPILING AND MINGW AND CMAKE_SYSTEM_PROCESSOR STREQUAL "x86_64")
  if(CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    set(cargo_platform x86_64-pc-windows-gnullvm)
  else()
    set(cargo_platform x86_64-pc-windows-gnu)
  endif()
endif()
if(cargo_platform)
  set(cargo_platform_args --target "${cargo_platform}")
  string(APPEND cargo_platform_dir "/${cargo_platform}")
endif()
set(library "${cargo_platform_dir}/release/${CMAKE_STATIC_LIBRARY_PREFIX}rustcity_godot${CMAKE_STATIC_LIBRARY_SUFFIX}")
set(bridge_include "${cargo_target}/cxxbridge")
set(engine_header "${bridge_include}/rustcity-godot/src/lib.rs.h")
set(view_header "${bridge_include}/rustcity-godot/src/view.rs.h")
file(GLOB_RECURSE rust_sources CONFIGURE_DEPENDS
  "${CMAKE_CURRENT_SOURCE_DIR}/src/*.rs"
  "${CMAKE_CURRENT_SOURCE_DIR}/../../../rustcity-core/src/*.rs")
add_custom_command(OUTPUT "${library}" "${engine_header}" "${view_header}"
  COMMAND "${RUSTCITY_CARGO}" build --locked --release --manifest-path "${manifest}"
          --target-dir "${cargo_target}" ${cargo_platform_args}
  DEPENDS ${rust_sources} "${manifest}" Cargo.lock build.rs
          "${CMAKE_CURRENT_SOURCE_DIR}/../../../rustcity-core/Cargo.toml"
  VERBATIM)
add_custom_target(rustcity_bridge DEPENDS "${library}" "${engine_header}" "${view_header}")
add_library(rustcity_godot INTERFACE)
add_dependencies(rustcity_godot rustcity_bridge)
target_include_directories(rustcity_godot INTERFACE "${bridge_include}")
target_link_libraries(rustcity_godot INTERFACE "${library}" Threads::Threads ${CMAKE_DL_LIBS})
if(WIN32)
  target_link_libraries(rustcity_godot INTERFACE advapi32 bcrypt ntdll userenv ws2_32)
endif()
