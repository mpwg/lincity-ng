find_package(SDL3 3.2.0 REQUIRED)
find_package(SDL3_image REQUIRED)
find_package(SDL3_mixer REQUIRED)
find_package(SDL3_ttf REQUIRED)

# find_package(LibXslt NO_CMAKE_FIND_ROOT_PATH)
# The libxslt find module is broken when cross compiling because it looks in the
# target environment instead of the host. The workaround is to find only the
# xsltproc program.
if(NOT TARGET LibXslt::xsltproc)
	find_program(LIBXSLT_XSLTPROC_EXECUTABLE NAMES xsltproc REQUIRED)
	if(LIBXSLT_XSLTPROC_EXECUTABLE)
		add_executable(LibXslt::xsltproc IMPORTED)
		set_target_properties(LibXslt::xsltproc PROPERTIES IMPORTED_LOCATION "${LIBXSLT_XSLTPROC_EXECUTABLE}")
	endif()
endif()


include(cmake/CheckFrameCounts.cmake)
add_subdirectory(src)
add_subdirectory(lib)
add_subdirectory(data "${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_APPDATADIR}")

### runtime dependency installation for Windows
set_target_properties(lincity-ng PROPERTIES RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_BINDIR})
install(TARGETS lincity-ng
	RUNTIME_DEPENDENCY_SET dll_dependencies
	RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)
if(WIN32)
	# from https://gitlab.kitware.com/cmake/cmake/-/issues/20753#note_870046
	if(MINGW AND CMAKE_CROSSCOMPILING)
		# workaround for crosscompiling on linux/mingw for windows
		install(CODE [[
			set(CMAKE_GET_RUNTIME_DEPENDENCIES_PLATFORM "windows+pe")
			set(CMAKE_GET_RUNTIME_DEPENDENCIES_TOOL "objdump")
			set(CMAKE_GET_RUNTIME_DEPENDENCIES_COMMAND "${CMAKE_OBJDUMP}")
			# file(WRITE "objdump_unix2dos.sh" "${CMAKE_OBJDUMP} $@ | unix2dos")
			# file(CHMOD "objdump_unix2dos.sh" PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE)
		]])
	endif()

	install(RUNTIME_DEPENDENCY_SET dll_dependencies
		DIRECTORIES ${CMAKE_FIND_ROOT_PATH}/bin  # dll's from mingw
		UNRESOLVED_DEPENDENCIES_VAR DUMMY  # hacky!
		RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
	)
endif()
# if (WIN32)
# 	install(FILES $<TARGET_PDB_FILE:lincity-ng> DESTINATION . OPTIONAL)
# endif()
