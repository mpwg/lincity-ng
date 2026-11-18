add_executable(core_units parity/reference_units.cpp)
target_include_directories(core_units PRIVATE "${PROJECT_SOURCE_DIR}/src/lincity")
target_link_libraries(core_units PRIVATE LinCity::Core)
add_test(NAME core.units COMMAND core_units)

add_executable(core_scenario scenario.cpp)
target_link_libraries(core_scenario PRIVATE LinCity::Core randutil)
foreach(scenario Beach Rocket_98 bad_times extreme_arid extreme_wetland good_times)
  add_test(NAME "core.scenario.${scenario}" COMMAND core_scenario
    "${PROJECT_SOURCE_DIR}/data/opening/${scenario}.scn.gz"
    "${CMAKE_CURRENT_BINARY_DIR}/${scenario}.sav.gz")
endforeach()

add_executable(core_actions actions.cpp)
target_link_libraries(core_actions PRIVATE LinCity::Core)
add_test(NAME core.actions COMMAND core_actions)

add_executable(core_animation animation.cpp)
target_link_libraries(core_animation PRIVATE LinCity::Core)
add_test(NAME core.animation COMMAND core_animation)
