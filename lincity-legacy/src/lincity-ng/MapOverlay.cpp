/* ---------------------------------------------------------------------- *
 * src/lincity/MapOverlay.cpp
 * This file is part of Lincity-NG.
 *
 * Copyright (C) 2005      David Kamphausen <david.kamphausen@web.de>
 * Copyright (C) 2024-2026 David Bears <dbear4q@gmail.com>
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

#include "MapOverlay.hpp"
#include "lincity/lin-city.hpp"
#include "lincity/modules/all_modules.hpp"
#include "lincity/transport.hpp"
#include "lincity/world.hpp"
#include <cassert>

static inline uint8_t brightness(const OverlayColor &c) {
  uint16_t x = c.r;
  x += c.g;
  x += c.b;
  return x / 3;
}

static inline OverlayColor makeGrey(const OverlayColor &c) {
  uint8_t b = brightness(c);
  return OverlayColor(b, b, b);
}

static inline OverlayColor makeBlue(const OverlayColor &c) {
  uint8_t b = brightness(c);
  return OverlayColor(0, 0, b);
}

static inline OverlayColor light(const OverlayColor &c, uint8_t b) {
  return OverlayColor(((uint16_t(c.r) * b) >> 8), ((uint16_t(c.g) * b) >> 8),
                      ((uint16_t(c.b) * b) >> 8));
}

OverlayColor MapOverlay::normal(const MapTile &tile) {
  int mc = tile.getConstructionGroup()->colour;

  int red = 0;
  int green = 0;
  int blue = 0;

  if (mc & 32) {
    red = 8 * (mc & 31);
  }
  if (mc & 64) {
    green = 8 * (mc & 31);
  }
  if (mc & 128) {
    blue = 8 * (mc & 31);
  }

  return OverlayColor(red, green, blue);
}

OverlayColor MapOverlay::colour(const World &world, const MapTile &tile,
                                DisplayMode mMode, Commodity stuff_ID) {

  switch (mMode) {
  case NORMAL:
    return normal(tile);
  case POLLUTION: {
    short pol = tile.pollution;
    float v = pol / 600.0;
    if (v < 0)
      v = 0;
    if (v > 1)
      v = 1;
    OverlayColor mc((int)(0xFF * v), (int)(0xFF * (1 - v)), 0);
    mc = light(mc, brightness(normal(tile)));
    return mc;
  }
  case FIRE:
    if (tile.getGroup() == GROUP_FIRE) {
      if (!static_cast<const Fire *>(tile.getReportingConstruction())
               ->smoking_days) {
        return OverlayColor(0xFF, 0, 0); // still burning (red)
      } else {
        return OverlayColor(0xFF, 0x99, 0); // unbulldozable (orange)
      }
    }
    if (tile.getConstructionGroup() == &fireStationConstructionGroup) {
      return OverlayColor(0, 0xFF, 0); // green
    }
    // fall through
  case CRICKET:
    if (tile.getConstructionGroup() == &cricketConstructionGroup &&
        mMode == CRICKET) {
      return OverlayColor(0, 0xFF, 0); // green
    }
    // fall through
  case HEALTH: {
    if (tile.getConstructionGroup() == &healthCentreConstructionGroup &&
        mMode == HEALTH) {
      return OverlayColor(0, 0xFF, 0); // green
    }
    int csttileflags =
        tile.getReportingConstruction()
            ? world.map(tile.getReportingConstruction()->point)->flags
            : tile.flags;
    if ((csttileflags & FLAG_FIRE_COVER) && mMode == FIRE ||
        (csttileflags & FLAG_CRICKET_COVER) && mMode == CRICKET ||
        (csttileflags & FLAG_HEALTH_COVER) && mMode == HEALTH) {
      OverlayColor mc(0, 0xFF, 0);
      mc = light(mc, brightness(normal(tile)));
      return mc;
    } else
      return makeGrey(normal(tile));
  }
  case UB40: {
    int job_level =
        tile.getReportingConstruction()
            ? tile.getReportingConstruction()->tellstuff(STUFF_LABOR, -1)
            : -1;
    if (job_level == -1) { // Not a "jobby" place at all
      return makeGrey(normal(tile));
    }
    if (tile.is_residence()) {
      if (job_level > 95 * TRANSPORT_QUANTA / 100)
        return OverlayColor(0xFF, 0, 0);
      else if (job_level > 90 * TRANSPORT_QUANTA / 100)
        return OverlayColor(0x7F, 0, 0);
      else
        return makeGrey(normal(tile));

    }

    else { // not a residence
      if (job_level < 5 * TRANSPORT_QUANTA / 100)
        return OverlayColor(0xFF, 0xFF, 0); // yellow
      else if (job_level < 10 * TRANSPORT_QUANTA / 100)
        return OverlayColor(0xFF, 0x99, 0); // orange
      else
        return makeGrey(normal(tile));
    }
  }
  case COAL: {
    if (!world.coal_survey_done) {
      return OverlayColor(0, 0, 0);
    }
    if (tile.coal_reserve == 0)
      return makeGrey(normal(tile));
    else if (tile.coal_reserve >= COAL_RESERVE_SIZE / 2)
      return OverlayColor(0, 0xFF, 0);
    else
      return OverlayColor(0xFF, 0, 0);
  }
  case STARVE: {
    int food_level =
        tile.getReportingConstruction()
            ? tile.getReportingConstruction()->tellstuff(STUFF_FOOD, -1)
            : 1;
    int water_level =
        tile.getReportingConstruction()
            ? tile.getReportingConstruction()->tellstuff(STUFF_WATER, -1)
            : 1;
    int crit_level = water_level < food_level ? water_level : food_level;
    // don't care about other eaters or drinkers
    if (tile.is_residence()) {
      if (crit_level < 5 * TRANSPORT_QUANTA / 100)
        return OverlayColor(0xFF, 0, 0);
      else if (crit_level < 10 * TRANSPORT_QUANTA / 100)
        return OverlayColor(0x7F, 0, 0);
      else
        return OverlayColor(0, 0xFF, 0);
    } else
      return makeGrey(normal(tile));
  }
  case POWER: {
    OverlayColor mc;

    mc = makeGrey(normal(tile));
    int lovolt_level =
        tile.getReportingConstruction()
            ? tile.getReportingConstruction()->tellstuff(STUFF_LOVOLT, -1)
            : -1;
    int hivolt_level =
        tile.getReportingConstruction()
            ? tile.getReportingConstruction()->tellstuff(STUFF_HIVOLT, -1)
            : -1;
    if (lovolt_level > -1 || hivolt_level > -1) {
      mc = OverlayColor(0xFF, 0, 0);
      if (lovolt_level > 5 * TRANSPORT_QUANTA / 100)
        mc = OverlayColor(0, 0x7F, 0);
      if (hivolt_level > 5 * TRANSPORT_QUANTA / 100)
        mc = OverlayColor(0, 0xFF, 0);
    }
    if (tile.getGroup() == GROUP_POWER_LINE)
      mc = OverlayColor(0xFF, 0xFF, 0); // yellow
    return mc;
  }
  case TRAFFIC: {
    int conflags = tile.getReportingConstruction()
                       ? tile.getReportingConstruction()->flags
                       : 0;
    if ((conflags & FLAG_IS_TRANSPORT) ||
        (tile.getGroup() == GROUP_POWER_LINE)) {
      float loc_lvl = -1;
      if (conflags & FLAG_IS_TRANSPORT) {
        const Transport *transport =
            static_cast<const Transport *>(tile.getReportingConstruction());
        if (transport->getConstructionGroup()
                ->commodityRuleCount[stuff_ID]
                .maxload)
          loc_lvl = transport->trafficCount[stuff_ID];
      } else if (tile.getGroup() == GROUP_POWER_LINE) {
        const Powerline *powerline =
            static_cast<const Powerline *>(tile.getReportingConstruction());
        if (powerline->getConstructionGroup()
                ->commodityRuleCount[stuff_ID]
                .maxload)
          loc_lvl = powerline->trafficCount[stuff_ID];
      }
      if (loc_lvl < 0)
        return makeGrey(normal(tile));
      loc_lvl = loc_lvl * TRANSPORT_RATE / TRANSPORT_QUANTA;

#ifdef DEBUG
      assert(!(loc_lvl > 0.928)); // should be limit from fluctuations
#endif
      loc_lvl = 2 - 2 / (1 + ((loc_lvl > 0.928) ? 1 : loc_lvl / 0.928));
      // more contrast for small loads
      OverlayColor mc((int)(0xFF * loc_lvl), (int)(0xFF * (1 - loc_lvl)), 0);
      return mc;
    } else { // not a Transport, make bluish if in range of a markt
      int csttileflags =
          tile.getReportingConstruction()
              ? world.map(tile.getReportingConstruction()->point)->flags
              : tile.flags;
      if ((csttileflags & FLAG_MARKET_COVER) &&
          marketConstructionGroup.commodityRuleCount[stuff_ID].maxload) {
        return makeBlue(normal(tile));
      }
      return makeGrey(normal(tile));
    }
  }
  case COMMODITIES: {
    float loc_lvl =
        tile.getReportingConstruction()
            ? tile.getReportingConstruction()->tellstuff(stuff_ID, -1)
            : -1;
    float red, green, blue;

    if (loc_lvl < 0)
      return makeGrey(normal(tile));
    loc_lvl /= TRANSPORT_QUANTA;
    if (stuff_ID == STUFF_WASTE) // so far waste is the only bad commodity
      loc_lvl = 1 - loc_lvl;
    red = 1 - 2.25 * loc_lvl;
    if (red < 0)
      red = 0;
    if (!tile.getReportingConstruction()
             ->getConstructionGroup()
             ->commodityRuleCount[stuff_ID]
             .take)
      red = 0;
    blue = 1 - 2.25 * (1 - loc_lvl);
    if (blue < 0)
      blue = 0;
    if (!tile.getReportingConstruction()
             ->getConstructionGroup()
             ->commodityRuleCount[stuff_ID]
             .give)
      blue = 0;
    green = 1 - red - blue;
    OverlayColor mc(static_cast<int>(0xFF * red),
                    static_cast<int>(0xFF * green),
                    static_cast<int>(0xFF * blue));
    return mc;
  }
  case MAX:
  default:
    std::cerr << "Undefined MiniMap-Display-type!" << std::endl;
  };
  // default coloor is a nasty looking pink
  return OverlayColor(0xFF, 0, 0xFF);
}
