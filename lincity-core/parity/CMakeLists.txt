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
