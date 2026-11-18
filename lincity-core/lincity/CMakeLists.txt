add_subdirectory(modules)

add_library(lincity_lib  STATIC
  all_buildings.hpp
  commodities.cpp
  commodities.hpp
  engine.cpp
  groups.hpp
  init_game.cpp
  init_game.hpp
  lin-city.hpp
  lintypes.cpp
  lintypes.hpp
  MapPoint.cpp
  MapPoint.hpp
  messages.cpp
  messages.hpp
  resources.cpp
  resources.hpp
  simulate.cpp
  stats.cpp
  stats.hpp
  sustainable.hpp
  transport.cpp
  transport.hpp
  UserOperation.cpp
  UserOperation.hpp
  Vehicles.cpp
  Vehicles.hpp
  world.cpp
  world.hpp
  xmlloadsave.cpp
  xmlloadsave.hpp
)

add_library(LinCity::Core ALIAS lincity_lib)
add_library(LinCity::lib ALIAS lincity_lib)
target_compile_features(lincity_lib PUBLIC cxx_std_17)
target_include_directories(lincity_lib PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/..")

target_link_libraries(lincity_lib
  PUBLIC
  LinCity::lib_modules
  ZLIB::ZLIB
  LibXml++-5.0::LibXml++-5.0
  LibXml2::LibXml2
  PRIVATE
  gettext
  xmlutil
  randutil
  fmt::fmt
)

include_directories(.)

if(COMMAND translate_target)
  translate_target(lincity_lib lincity_lib.pot)
endif()
