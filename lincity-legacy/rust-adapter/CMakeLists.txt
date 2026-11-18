find_program(RUSTCITY_CARGO cargo REQUIRED)
set(manifest "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml")
set(cargo_target "${CMAKE_CURRENT_BINARY_DIR}/cargo")
set(library "${cargo_target}/release/${CMAKE_STATIC_LIBRARY_PREFIX}rustcity_legacy${CMAKE_STATIC_LIBRARY_SUFFIX}")
set(bridge_include "${cargo_target}/cxxbridge")
set(engine_header "${bridge_include}/rustcity-legacy/src/lib.rs.h")
set(view_header "${bridge_include}/rustcity-legacy/src/view.rs.h")
file(GLOB_RECURSE rust_sources CONFIGURE_DEPENDS
  "${CMAKE_CURRENT_SOURCE_DIR}/src/*.rs"
  "${CMAKE_CURRENT_SOURCE_DIR}/../../rustcity-core/src/*.rs")
add_custom_command(OUTPUT "${library}" "${engine_header}" "${view_header}"
  COMMAND "${RUSTCITY_CARGO}" build --locked --release --manifest-path "${manifest}"
          --target-dir "${cargo_target}"
  DEPENDS ${rust_sources} "${manifest}" Cargo.lock build.rs
          "${CMAKE_CURRENT_SOURCE_DIR}/../../rustcity-core/Cargo.toml"
  VERBATIM)
add_custom_target(rustcity_bridge DEPENDS "${library}" "${engine_header}" "${view_header}")
add_library(rustcity_desktop STATIC RustDesktopWorld.cpp projection.cpp)
add_dependencies(rustcity_desktop rustcity_bridge)
target_compile_features(rustcity_desktop PRIVATE cxx_std_20)
target_compile_definitions(rustcity_desktop PUBLIC LINCITY_RUST_BACKEND)
target_include_directories(rustcity_desktop PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}" "${bridge_include}"
  "${CMAKE_CURRENT_SOURCE_DIR}/../src/lincity-ng")
target_link_libraries(rustcity_desktop PUBLIC LinCity::Core "${library}" Threads::Threads ${CMAKE_DL_LIBS})
if(BUILD_TESTING)
  add_executable(rustcity_desktop_lifecycle lifecycle.cpp)
  target_link_libraries(rustcity_desktop_lifecycle PRIVATE rustcity_desktop)
  add_test(NAME rustcity.desktop.lifecycle COMMAND rustcity_desktop_lifecycle
    "${CMAKE_CURRENT_SOURCE_DIR}/../../lincity-core/tests/fixtures/blacksmith-2140.xml"
    "${CMAKE_CURRENT_BINARY_DIR}/resumed-city.gz")

  add_executable(rustcity_desktop_projection_test projection_test.cpp)
  target_link_libraries(rustcity_desktop_projection_test PRIVATE rustcity_desktop)
  add_test(NAME rustcity.desktop.projection COMMAND rustcity_desktop_projection_test
    "${CMAKE_CURRENT_SOURCE_DIR}/tests/projection.xml")
endif()
