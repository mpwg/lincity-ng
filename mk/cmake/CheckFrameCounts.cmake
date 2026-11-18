# Keep the desktop images and the standalone core animation lengths in sync.
find_package(Python3 REQUIRED COMPONENTS Interpreter)
set(tile_manifest "${CMAKE_CURRENT_LIST_DIR}/../../data/images/tiles/images.xml")
set(frame_generator "${CMAKE_CURRENT_LIST_DIR}/generate_frame_counts.py")
set(core_frames "${CMAKE_CURRENT_LIST_DIR}/../../lincity-core/lincity/frame_counts.inc")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
  "${tile_manifest}" "${frame_generator}" "${core_frames}")
execute_process(COMMAND "${Python3_EXECUTABLE}" "${frame_generator}" "${tile_manifest}"
  OUTPUT_VARIABLE frame_counts COMMAND_ERROR_IS_FATAL ANY)
file(READ "${core_frames}" embedded_frame_counts)
string(REPLACE "\r\n" "\n" embedded_frame_counts "${embedded_frame_counts}")
if(NOT frame_counts STREQUAL embedded_frame_counts)
  message(FATAL_ERROR
    "Tile animation lengths changed. Regenerate ${core_frames} using ${frame_generator} and ${tile_manifest}.")
endif()
