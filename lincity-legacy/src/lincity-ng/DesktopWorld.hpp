// Backend selection for the legacy desktop. The C++ build uses the original core.
#pragma once

#include <stdexcept>
#include "lincity/UserOperation.hpp"
#include "lincity/init_game.hpp"
#include "lincity/world.hpp"

enum class DesktopClimate { River, Desert, Temperate, Swamp };

#ifdef LINCITY_RUST_BACKEND
#include "RustDesktopWorld.hpp"
#else
using DesktopWorld = World;

inline std::unique_ptr<World> generateDesktopCity(DesktopClimate climate, city_settings* settings, int side) {
  switch(climate) {
    case DesktopClimate::River: return new_city(settings, side);
    case DesktopClimate::Desert: return new_desert_city(settings, side);
    case DesktopClimate::Temperate: return new_temperate_city(settings, side);
    case DesktopClimate::Swamp: return new_swamp_city(settings, side);
  }
  throw std::invalid_argument("unknown terrain climate");
}
inline bool desktopPreview(const World& w, const UserOperation& op, MapPoint p, Message::ptr& why) {
  return op.isAllowedHere(w, p, why);
}
inline void desktopApply(World& w, const UserOperation& op, MapPoint p) {
  op.execute(w, p);
}
inline ConstructionGroup* desktopBuildGroup(const World& w, const UserOperation& op) {
  return op.actualConstructionGroup(w);
}
inline bool desktopAllowed(const World& w, const UserOperation& op, Message::ptr& why) {
  return op.isAllowed(w, why);
}
inline int desktopCost(const World& w, const UserOperation& op, MapPoint p = {}) {
  return op.cost(w, p);
}
inline void desktopFocus(World& w, MapPoint p) { w.map.recentPoint = p; }
inline void desktopHistory(World& w, std::size_t months) { w.stats.ensurePopulationHistory(months); }
#endif
