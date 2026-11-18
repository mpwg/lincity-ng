add_executable(lincity-ng
  ButtonPanel.cpp
  ButtonPanel.hpp
  Config.cpp
  Config.hpp
  Dialog.cpp
  Dialog.hpp
  DisplayFormat.cpp
  DisplayFormat.hpp
  EconomyGraph.cpp
  EconomyGraph.hpp
  Game.cpp
  Game.hpp
  GameView.cpp
  GameView.hpp
  HelpWindow.cpp
  HelpWindow.hpp
  main.cpp
  main.hpp
  MainLincity.cpp
  MainLincity.hpp
  MainMenu.cpp
  MainMenu.hpp
  MiniMap.cpp
  MiniMap.hpp
  Mps.cpp
  Mps.hpp
  PBar.cpp
  PBar.hpp
  ScreenInterface.cpp
  Sound.cpp
  Sound.hpp
  TimerInterface.cpp
  TimerInterface.hpp
  Util.cpp
  Util.hpp
  Video.cpp
  Video.hpp
)

target_link_libraries(lincity-ng
  PUBLIC
  LinCity::Core
  LinCity::gui
  $<IF:$<TARGET_EXISTS:SDL3::SDL3>,SDL3::SDL3,SDL3::SDL3-static>
  $<IF:$<TARGET_EXISTS:SDL3_mixer::SDL3_mixer>,SDL3_mixer::SDL3_mixer,SDL3_mixer::SDL3_mixer-static>
  PRIVATE
  cfgpath
  gettextutil
  $<TARGET_NAME_IF_EXISTS:SDL3::SDL3main>
  $<IF:$<TARGET_EXISTS:SDL3_image::SDL3_image>,SDL3_image::SDL3_image,SDL3_image::SDL3_image-static>
  fmt::fmt
  xmlutil
  ptrutil
)

include_directories(.)

include(TranslateTarget)
translate_target(lincity-ng lincity-ng.pot)


if(LINCITY_CORE_BACKEND STREQUAL "rust")
  add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../../rust-adapter" rust-adapter)
  target_link_libraries(lincity-ng PRIVATE rustcity_desktop)
endif()
