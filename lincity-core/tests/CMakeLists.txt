add_executable(core_units reference_units.cpp)
target_include_directories(core_units PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../lincity")
target_link_libraries(core_units PRIVATE LinCity::Core)
add_test(NAME core.units COMMAND core_units)

add_executable(core_scenario scenario.cpp)
target_link_libraries(core_scenario PRIVATE LinCity::Core randutil)
foreach(scenario Beach Rocket_98 bad_times extreme_arid extreme_wetland good_times)
  add_test(NAME "core.scenario.${scenario}" COMMAND core_scenario
    "${CMAKE_CURRENT_SOURCE_DIR}/../opening/${scenario}.scn.gz"
    "${CMAKE_CURRENT_BINARY_DIR}/${scenario}.sav.gz")
endforeach()

add_executable(core_actions actions.cpp)
target_link_libraries(core_actions PRIVATE LinCity::Core)
add_test(NAME core.actions COMMAND core_actions)

add_executable(core_animation animation.cpp)
target_link_libraries(core_animation PRIVATE LinCity::Core)
add_test(NAME core.animation COMMAND core_animation)

add_executable(core_modules modules.cpp)
target_link_libraries(core_modules PRIVATE LinCity::Core randutil)
add_test(NAME core.modules COMMAND core_modules)

add_executable(core_world_generation world_generation.cpp)
target_link_libraries(core_world_generation PRIVATE LinCity::Core randutil)
add_test(NAME core.world_generation COMMAND core_world_generation)

add_executable(core_replay replay/main.cpp replay/events.cpp)
target_link_libraries(core_replay PRIVATE LinCity::Core randutil)

add_executable(core_characterization characterization.cpp)
target_link_libraries(core_characterization PRIVATE LinCity::Core randutil)
foreach(case port vehicles calendar fire windmill saves)
  add_test(NAME "core.characterization.${case}" COMMAND core_characterization "${case}"
    "${CMAKE_CURRENT_SOURCE_DIR}/fixtures" "${CMAKE_CURRENT_BINARY_DIR}/${case}.sav.gz")
endforeach()

find_package(Python3 COMPONENTS Interpreter QUIET)
if(Python3_Interpreter_FOUND)
  foreach(case small scenario)
    add_test(NAME "core.replay.${case}" COMMAND "${Python3_EXECUTABLE}" -B
      "${CMAKE_CURRENT_SOURCE_DIR}/replay/check.py" $<TARGET_FILE:core_replay> "${case}")
  endforeach()
  add_test(NAME core.replay.traffic COMMAND "${Python3_EXECUTABLE}" -B
    "${CMAKE_CURRENT_SOURCE_DIR}/replay/check.py" $<TARGET_FILE:core_replay> scenario --profile traffic)
  if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    find_library(LINCITY_JEMALLOC NAMES jemalloc libjemalloc.so.2)
    if(LINCITY_JEMALLOC)
      add_test(NAME core.replay.allocator COMMAND "${Python3_EXECUTABLE}" -B
        "${CMAKE_CURRENT_SOURCE_DIR}/replay/check.py" $<TARGET_FILE:core_replay>
        scenario --allocator "${LINCITY_JEMALLOC}")
    endif()
  endif()
endif()
