# Embed the animation lengths so the simulation can run without installed assets.
find_package(Python3 REQUIRED COMPONENTS Interpreter)
set(tile_manifest "${CMAKE_CURRENT_LIST_DIR}/../../data/images/tiles/images.xml")
set(frame_generator "${CMAKE_CURRENT_LIST_DIR}/generate_frame_counts.py")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
  "${tile_manifest}" "${frame_generator}")
execute_process(COMMAND "${Python3_EXECUTABLE}" "${frame_generator}" "${tile_manifest}"
  OUTPUT_VARIABLE frame_counts COMMAND_ERROR_IS_FATAL ANY)
file(GENERATE OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/frame_counts.inc" CONTENT "${frame_counts}")
