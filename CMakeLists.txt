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
  LANGUAGES CXX
  DESCRIPTION "A city simulation game"
  HOMEPAGE_URL "https://github.com/lincity-ng/lincity-ng"
)

set(CMAKE_POSITION_INDEPENDENT_CODE ON)
find_package(Threads REQUIRED)
find_package(fmt 9.0.0 REQUIRED)

add_subdirectory(godot/native)
