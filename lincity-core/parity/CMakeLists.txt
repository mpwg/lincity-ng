add_executable(core_parity_reference reference.cpp)
target_link_libraries(core_parity_reference PRIVATE LinCity::Core)
set(case_dir "${CMAKE_CURRENT_SOURCE_DIR}/cases/v1")
add_test(NAME core.parity.cpp.v1 COMMAND "${CMAKE_COMMAND}"
  "-DPROGRAM=$<TARGET_FILE:core_parity_reference>" "-DCASES=${case_dir}"
  -P "${CMAKE_CURRENT_SOURCE_DIR}/compare.cmake")
