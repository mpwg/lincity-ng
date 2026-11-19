// Shared colours for the desktop map and minimap overlays.
#pragma once

#include "lincity/commodities.hpp"
#include <cstdint>

class World;
class MapTile;

struct OverlayColor {
  std::uint8_t r, g, b;
  OverlayColor(int r = 0, int g = 0, int b = 0) : r(r), g(g), b(b) {}
};

struct MapOverlay {
  enum DisplayMode {
    NORMAL,
    POLLUTION,
    UB40,
    STARVE,
    POWER,
    FIRE,
    CRICKET,
    HEALTH,
    COAL,
    TRAFFIC,
    COMMODITIES,
    MAX
  };
  static OverlayColor normal(const MapTile &tile);
  static OverlayColor colour(const World &world, const MapTile &tile,
                             DisplayMode mode, Commodity commodity);
};
