include(FetchContent)
set(GODOTCPP_DISABLE_EXCEPTIONS OFF CACHE BOOL "The simulation reports failed operations with exceptions" FORCE)
set(GODOTCPP_BUILD_PROFILE "${CMAKE_CURRENT_SOURCE_DIR}/bindings.json" CACHE FILEPATH "Godot bindings used by CityWorld")
FetchContent_Declare(godot_cpp
  GIT_REPOSITORY https://github.com/godotengine/godot-cpp.git
  GIT_TAG e83fd0904c13356ed1d4c3d09f8bb9132bdc6b77 # godot-4.5-stable
)
FetchContent_MakeAvailable(godot_cpp)
add_library(lincity_godot SHARED city_world.cpp
  "${PROJECT_SOURCE_DIR}/lincity-legacy/src/lincity-ng/MapOverlay.cpp")
target_link_libraries(lincity_godot PRIVATE godot::cpp LinCity::Core fmt::fmt)
target_include_directories(lincity_godot PRIVATE "${PROJECT_SOURCE_DIR}/lincity-legacy/src/lincity-ng")
if(LINCITY_CORE_BACKEND STREQUAL "rust")
  target_link_libraries(lincity_godot PRIVATE rustcity_desktop)
endif()
set(LINCITY_GODOT_OUTPUT_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../bin" CACHE PATH "Output directory for the Godot extension")
set_target_properties(lincity_godot PROPERTIES
  LIBRARY_OUTPUT_DIRECTORY "$<1:${LINCITY_GODOT_OUTPUT_DIR}>"
  RUNTIME_OUTPUT_DIRECTORY "$<1:${LINCITY_GODOT_OUTPUT_DIR}>"
  PREFIX "lib"
  CXX_VISIBILITY_PRESET hidden)
