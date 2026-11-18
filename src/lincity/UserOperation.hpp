/* ---------------------------------------------------------------------- *
 * src/lincity/UserOperation.hpp
 * This file is part of Lincity-NG.
 *
 * Copyright (C) 1995-1997 I J Peters
 * Copyright (C) 1997-2005 Greg Sharp
 * Copyright (C) 2000-2004 Corey Keasling
 * Copyright (C) 2025      David Bears
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

#ifndef USER_OPERATION_H__
#define USER_OPERATION_H__

#include "MapPoint.hpp"
#include "messages.hpp"  // for Message

class ConstructionGroup;
class World;

class UserOperation {
public:
  enum Action {
    ACTION_QUERY,
    ACTION_EVACUATE,
    ACTION_BULLDOZE,
    ACTION_FLOOD,
    ACTION_BUILD,
    ACTION_UNKNOWN
  };
  Action action = ACTION_QUERY;
  ConstructionGroup *constructionGroup = nullptr; // non-null for ACTION_BUILD

  bool isAllowedHere(const World& world, MapPoint point, Message::ptr& message) const;
  bool isAllowed(const World& world, Message::ptr& message) const;
  void execute(World& world, MapPoint point) const;
  ConstructionGroup *actualConstructionGroup(
    const World& world, MapPoint point = MapPoint()) const;
  int cost(const World& world, MapPoint point = MapPoint()) const;
};

#endif
