/* ---------------------------------------------------------------------- *
 * src/lincity/resources.hpp
 * This file is part of Lincity-NG.
 *
 * Copyright (C) 2022-2024 David Bears <dbear4q@gmail.com>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
** ---------------------------------------------------------------------- */

#ifndef __LINCITY_RESOURCES_HPP__
#define __LINCITY_RESOURCES_HPP__

#include <map>
#include <string>

// Logical animation resources shared by the simulation and its frontends.
class ResourceGroup {
public:
    explicit ResourceGroup(const std::string& tag);
    const std::string resourceID;
    const int frameCount;
    bool is_vehicle = false;
    static std::map<std::string, ResourceGroup*> resMap;
};

struct ExtraFrame{
    ExtraFrame(void){
        move_x = 0;
        move_y = 0;
        frame = 0;
        resourceGroup = 0;
    }

    int move_x; // >0 moves frame to the right
    int move_y; // >0 moves frame downwards
    int frame; //frame >= 0 will be rendered as overlay
    const ResourceGroup *resourceGroup;
};

#endif  // __LINCITY_RESOURCES_HPP__
