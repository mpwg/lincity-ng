## ---------------------------------------------------------------------- ##
 # CMakeLists.txt
 # This file is part of Lincity-NG.
 #
 # Copyright (C) 2023      Edu Garcia
 # Copyright (C) 2023-2025 David Bears <dbear4q@gmail.com>
 #
 # This program is free software; you can redistribute it and/or modify
 # it under the terms of the GNU General Public License as published by
 # the Free Software Foundation; either version 2 of the License, or
 # (at your option) any later version.
 #
 # This program is distributed in the hope that it will be useful,
 # but WITHOUT ANY WARRANTY; without even the implied warranty of
 # MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 # GNU General Public License for more details.
 #
 # You should have received a copy of the GNU General Public License along
 # with this program; if not, write to the Free Software Foundation, Inc.,
 # 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
## ---------------------------------------------------------------------- ##

cmake_minimum_required(VERSION 3.21)

project(lincity-ng
  VERSION 2.15.0
  LANGUAGES CXX C
  DESCRIPTION "A city simulation game"
  HOMEPAGE_URL "https://github.com/lincity-ng/lincity-ng"
)
set(PROJECT_NAME_PRETTY "LinCity-NG")
set(APPSTREAM_ID "io.github.lincity_ng.lincity-ng")


option(LINCITY_BUILD_GUI "Build the SDL desktop game" ON)
set(LINCITY_CORE_BACKEND "cpp" CACHE STRING "Simulation core for the legacy SDL interface (cpp or rust)")
set_property(CACHE LINCITY_CORE_BACKEND PROPERTY STRINGS cpp rust)
if(NOT LINCITY_CORE_BACKEND MATCHES "^(cpp|rust)$")
  message(FATAL_ERROR "LINCITY_CORE_BACKEND must be cpp or rust")
endif()
include(CTest)

### dependencies
list(APPEND CMAKE_MODULE_PATH
  ${CMAKE_SOURCE_DIR}/tooling/cmake/modules
  ${CMAKE_CURRENT_SOURCE_DIR}/lincity-core/cmake
)
set(CMAKE_FIND_PACKAGE_PREFER_CONFIG TRUE)
find_package(Threads)
find_package(ZLIB 1.0 REQUIRED)
find_package(LibXml2 REQUIRED) # requires >=2.6.11
find_package(fmt 9.0.0 REQUIRED)
find_package(LibXml++-5.0 REQUIRED)
find_package(Gettext)
find_package(Intl)
find_program(GETTEXT_XGETTEXT NAMES xgettext)
find_program(INCLUDE_WHAT_YOU_USE NAMES include-what-you-use)
find_package(Git)
### build configurations
set(BUILD_TYPES Release MinSizeRel RelWithDebInfo BetaTest Debug DebugOpt None)
set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS ${BUILD_TYPES})
string(TOUPPER "${BUILD_TYPES}" BUILD_TYPES_UPPER)
string(TOUPPER "${CMAKE_BUILD_TYPE}" BUILD_TYPE_UPPER)
if(NOT CMAKE_BUILD_TYPE)
	set(CMAKE_BUILD_TYPE Release)
	message(STATUS "No build type specified, defaulting to ${CMAKE_BUILD_TYPE}.")
elseif(NOT BUILD_TYPE_UPPER IN_LIST BUILD_TYPES_UPPER)
  # Make this an error eventually
  message(WARNING "Unrecognized build type: ${CMAKE_BUILD_TYPE}."
    " Use of unrecognized build types is deprecated."
    " See docs/BUILD_OPTIONS.md for a list of supported build types."
  )
	set(CMAKE_BUILD_TYPE None)
endif()
set(CMAKE_CXX_FLAGS_RELEASE        "-O3 -w -DNDEBUG")
set(CMAKE_CXX_FLAGS_MINSIZEREL     "-Os -w -DNDEBUG")
set(CMAKE_CXX_FLAGS_RELWITHDEBINFO "-O3 -g -DNDEBUG")
set(CMAKE_CXX_FLAGS_BETATEST       "-O3 -g -w")
set(CMAKE_CXX_FLAGS_DEBUG          "-O0 -g -DDEBUG")
set(CMAKE_CXX_FLAGS_DEBUGOPT       "-O2 -g -DDEBUG")
set(CMAKE_CXX_FLAGS_NONE           "")
foreach(BType ${BUILD_TYPES})
	string(TOUPPER ${BType} BTYPE)
	set(CMAKE_CXX_FLAGS_${BTYPE} ${CMAKE_CXX_FLAGS_${BTYPE}}
		CACHE STRING "Flags used by the CXX compiler for ${BTYPE} builds.")
	set(CMAKE_C_FLAGS_${BCONFIG} ${CMAKE_CXX_FLAGS_${BTYPE}}
		CACHE STRING "Flags used by the C compiler for ${BTYPE} builds.")
endforeach()
set(VERSION_RELEASE "")
set(VERSION_MINSIZEREL "")
set(VERSION_RELWITHDEBINFO "debinfo")
set(VERSION_BETATEST "beta")
set(VERSION_DEBUG "debug")
set(VERSION_DEBUGOPT "debug")
set(VERSION_NONE "")


### options
set(ENABLE_NLS "YES" CACHE BOOL "Enable native language support")
if(ENABLE_NLS AND NOT (Intl_FOUND AND Gettext_FOUND))
  message(WARNING "Native language support requires gettext and libintl, but"
    " one or both are missing. To suppress this warning, explicitly disable"
    " native language support with `-DENABLE_NLS=NO`.")
endif()

set(RELOCATABLE_DEFAULT NO)
if(WIN32)
  set(RELOCATABLE_DEFAULT YES)
endif()
set(LINCITYNG_RELOCATABLE ${RELOCATABLE_DEFAULT} CACHE BOOL "Build a relocatable binary")


### compilation options
set(CMAKE_CXX_STANDARD 17 CACHE STRING "C++ standard")
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
add_compile_options("-fmacro-prefix-map=${CMAKE_SOURCE_DIR}=.")
if(UNIX)
	add_compile_definitions("UNIX")
endif()
if(WIN32)
	add_compile_definitions("WIN32")
endif()
if(APPLE)
	add_compile_definitions("APPLE")
endif()

include(CheckCSourceCompiles)
set(CMAKE_REQUIRED_LIBRARIES ${LibIntl_LIBRARIES})
check_c_source_compiles("\
	extern int _nl_msg_cat_cntr; \
	int main() { \
		++_nl_msg_cat_cntr; \
		return 0; \
	} \
	"
	HAVE_NL_MSG_CAT_CNTR
)


### include what you use
if(NOT CMAKE_CROSSCOMPILING AND INCLUDE_WHAT_YOU_USE AND
  NOT CMAKE_CXX_FLAGS_${BUILD_TYPE_UPPER} MATCHES "(^| )-w( |$)"
)
  set(CMAKE_CXX_INCLUDE_WHAT_YOU_USE ${INCLUDE_WHAT_YOU_USE}
    -w
    -Xiwyu "--mapping_file=${CMAKE_SOURCE_DIR}/tooling/cmake/iwyu.map"
    -Xiwyu --verbose=3
  )
  set(CMAKE_C_INCLUDE_WHAT_YOU_USE ${CMAKE_CXX_INCLUDE_WHAT_YOU_USE})
endif()


### compute LinCity-NG version
set(LINCITYNG_VERSION_SUFFIX "" CACHE STRING
	"Custom version suffix to use; mostly useful for downstream packaging."
)
include(GetGitRevisionDescription)
if(DEFINED LINCITYNG_VERSION_SUFFIX AND NOT (LINCITYNG_VERSION_SUFFIX STREQUAL ""))
	set(FULL_PROJECT_VERSION "${PROJECT_VERSION}-${LINCITYNG_VERSION_SUFFIX}")
else()
	git_describe_working_tree(FULL_PROJECT_VERSION --match lincity-ng-*)
	if(FULL_PROJECT_VERSION)
		string(SUBSTRING ${FULL_PROJECT_VERSION} 11 -1 FULL_PROJECT_VERSION)
	else()
		set(FULL_PROJECT_VERSION "${PROJECT_VERSION}-unknown")
	endif()
endif()
if(VERSION_${BUILD_TYPE_UPPER})
  set(FULL_PROJECT_VERSION
    "${FULL_PROJECT_VERSION}-${VERSION_${BUILD_TYPE_UPPER}}"
  )
endif()
message(STATUS "LinCity-NG version: ${FULL_PROJECT_VERSION}")


### compute installation directories
include(GNUInstallDirs)
set(CMAKE_INSTALL_APPDATADIR ${CMAKE_INSTALL_DATADIR}/${CMAKE_PROJECT_NAME})
set(CMAKE_INSTALL_FULL_APPDATADIR ${CMAKE_INSTALL_FULL_DATADIR}/${CMAKE_PROJECT_NAME})
set(CMAKE_BINARY_BINDIR ${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_BINDIR})
set(CMAKE_BINARY_DATADIR ${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_DATADIR})
set(CMAKE_BINARY_APPDATADIR ${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_APPDATADIR})


### configure subdirectories
if(LINCITY_BUILD_GUI)
  include(TranslateTarget)
endif()
add_subdirectory(lincity-core)
if(BUILD_TESTING)
  add_subdirectory(tooling/regression)
endif()
if(LINCITY_BUILD_GUI)
  add_subdirectory(lincity-legacy)
  add_subdirectory(tooling/save-converter)
  add_subdirectory(docs ${CMAKE_INSTALL_DOCDIR})
  include(tooling/cmake/PackageDesktop.cmake)
endif()
