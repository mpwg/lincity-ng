add_executable(core_parity_reference reference.cpp)
target_link_libraries(core_parity_reference PRIVATE LinCity::Core)
set(case_dir "${CMAKE_CURRENT_SOURCE_DIR}/cases/v1")
add_test(NAME core.parity.cpp.v1 COMMAND "${CMAKE_COMMAND}"
  "-DPROGRAM=$<TARGET_FILE:core_parity_reference>" "-DCASES=${case_dir}"
  -P "${CMAKE_CURRENT_SOURCE_DIR}/compare.cmake")

set(rust_catalog "${CMAKE_CURRENT_SOURCE_DIR}/../../rustcity-core/src/building_kind/reference.tsv")
if(EXISTS "${rust_catalog}")
  add_executable(core_catalog_reference catalog_reference.cpp)
  target_link_libraries(core_catalog_reference PRIVATE LinCity::Core)
  add_test(NAME core.parity.catalog COMMAND "${CMAKE_COMMAND}"
    "-DPROGRAM=$<TARGET_FILE:core_catalog_reference>" "-DEXPECTED=${rust_catalog}"
    -P "${CMAKE_CURRENT_SOURCE_DIR}/compare.cmake")
endif()

set(rust_production_cases "${CMAKE_CURRENT_SOURCE_DIR}/../../rustcity-core/src/production/cases")
if(EXISTS "${rust_production_cases}/input.txt")
  add_executable(core_production_reference production_reference.cpp)
  target_link_libraries(core_production_reference PRIVATE LinCity::Core)
  add_test(NAME core.parity.production COMMAND "${CMAKE_COMMAND}"
    "-DPROGRAM=$<TARGET_FILE:core_production_reference>" "-DCASES=${rust_production_cases}"
    -P "${CMAKE_CURRENT_SOURCE_DIR}/compare.cmake")
endif()

set(rust_transport "${CMAKE_CURRENT_SOURCE_DIR}/../../rustcity-core/src/city/transport/reference.txt")
if(EXISTS "${rust_transport}")
  add_executable(core_transport_reference transport_reference.cpp)
  target_link_libraries(core_transport_reference PRIVATE LinCity::Core)
  add_test(NAME core.parity.transport COMMAND "${CMAKE_COMMAND}"
    "-DPROGRAM=$<TARGET_FILE:core_transport_reference>" "-DEXPECTED=${rust_transport}"
    -P "${CMAKE_CURRENT_SOURCE_DIR}/compare.cmake")
endif()

add_executable(core_generation_reference EXCLUDE_FROM_ALL generation_reference.cpp)
target_link_libraries(core_generation_reference PRIVATE LinCity::Core randutil)

# Cross-language tests are available when both source trees and Cargo are present.
# The standalone C++ core can still be copied and built independently.
set(rust_manifest "${CMAKE_CURRENT_SOURCE_DIR}/../../rustcity-core/Cargo.toml")
if(EXISTS "${rust_manifest}")
  find_program(CARGO_EXECUTABLE cargo)
  find_package(Python3 COMPONENTS Interpreter QUIET)
  if(CARGO_EXECUTABLE AND Python3_Interpreter_FOUND)
    set_property(TARGET core_generation_reference PROPERTY EXCLUDE_FROM_ALL FALSE)
    set(rust_target_dir "${CMAKE_CURRENT_BINARY_DIR}/cargo")
    set(rust_replay "${rust_target_dir}/release/rustcity-replay${CMAKE_EXECUTABLE_SUFFIX}")
    add_test(NAME rustcity.cross.build COMMAND "${CARGO_EXECUTABLE}" build
      --manifest-path "${rust_manifest}" --target-dir "${rust_target_dir}"
      --locked --release --bin rustcity-replay)
    set_tests_properties(rustcity.cross.build PROPERTIES FIXTURES_SETUP rustcity_replay)
    foreach(cross_case continuation traffic)
      add_test(NAME "rustcity.cross.${cross_case}" COMMAND "${Python3_EXECUTABLE}" -B
        "${CMAKE_CURRENT_SOURCE_DIR}/../tests/replay/check_continuation.py"
        "$<TARGET_FILE:core_replay>" "${rust_replay}" "${cross_case}")
      set_tests_properties("rustcity.cross.${cross_case}" PROPERTIES FIXTURES_REQUIRED rustcity_replay)
    endforeach()
    add_test(NAME rustcity.cross.generation COMMAND "${CMAKE_COMMAND}" -E env
      "RUSTCITY_CPP_GENERATOR=$<TARGET_FILE:core_generation_reference>"
      "${CARGO_EXECUTABLE}" test --manifest-path "${rust_manifest}"
      --target-dir "${rust_target_dir}" --locked --release --test world_generation -- --include-ignored)
    set_tests_properties(rustcity.cross.generation PROPERTIES FIXTURES_REQUIRED rustcity_replay)
  endif()
endif()

add_executable(core_building_report_reference building_report_reference.cpp)
target_link_libraries(core_building_report_reference PRIVATE LinCity::Core)
add_test(NAME core.parity.building_reports COMMAND "${CMAKE_COMMAND}"
  "-DPROGRAM=$<TARGET_FILE:core_building_report_reference>"
  "-DEXPECTED=${CMAKE_CURRENT_SOURCE_DIR}/../../rustcity-core/src/city/report/reference.tsv"
  -P "${CMAKE_CURRENT_SOURCE_DIR}/compare.cmake")
