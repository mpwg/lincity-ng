/* ---------------------------------------------------------------------- *
 * src/lincity/resources.cpp
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

#include "resources.hpp"

#include <memory>

std::map<std::string, ResourceGroup *> ResourceGroup::resMap;

namespace {
struct ResourceRegistryCleanup {
    ~ResourceRegistryCleanup() {
        for(const auto& entry : ResourceGroup::resMap)
            delete entry.second;
    }
};

ResourceRegistryCleanup resourceRegistryCleanup;

int frameCountFor(const std::string& tag) {
    static const std::map<std::string, int> counts = {
#include "frame_counts.inc"
    };
    const auto it = counts.find(tag);
    return it == counts.end() ? 0 : it->second;
}
}

ResourceGroup::ResourceGroup(const std::string& tag) :
    resourceID(tag), frameCount(frameCountFor(tag)) {}

ResourceGroup& ResourceGroup::registerGroup(const std::string& tag) {
    std::unique_ptr<ResourceGroup> group(new ResourceGroup(tag));
    auto entry = resMap.try_emplace(tag, nullptr).first;
    auto& registered = entry->second;
    if(!registered) registered = group.release();
    return *registered;
}
