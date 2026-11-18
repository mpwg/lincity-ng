#pragma once

#include "lincity/world.hpp"
#include "lincity/UserOperation.hpp"
#include "rustcity-legacy/src/lib.rs.h"
#include "projection.hpp"

enum class DesktopClimate;
struct _CitySettings;

class DesktopWorld final : public World {
public:
  explicit DesktopWorld(rust::Box<rustcity::Engine> engine);
  static std::unique_ptr<DesktopWorld> load(const std::filesystem::path&);
  void save(const std::filesystem::path&) const;
  void do_time_step();
  void do_animate(unsigned long milliseconds);
  void do_coal_survey();
  bool setMarketTrade(MapPoint, Commodity, bool buy, bool sell);
  bool setPortTrade(MapPoint, Commodity, bool buy, bool sell);
  bool launchRocket(MapPoint);
  void refresh(bool simulationChanged);
  rustcity::Engine& engine() { return *engine_; }
  const rustcity::Engine& engine() const { return *engine_; }
private:
  rust::Box<rustcity::Engine> engine_;
  RustViewProjection projection_;
};

std::unique_ptr<DesktopWorld> generateDesktopCity(DesktopClimate, _CitySettings*, int side);
bool desktopPreview(const DesktopWorld&, const UserOperation&, MapPoint, Message::ptr&);
bool desktopAllowed(const DesktopWorld&, const UserOperation&, Message::ptr&);
ConstructionGroup* desktopBuildGroup(const DesktopWorld&, const UserOperation&);
int desktopCost(const DesktopWorld&, const UserOperation&, MapPoint);
int desktopCost(const DesktopWorld&, const UserOperation&);
void desktopApply(DesktopWorld&, const UserOperation&, MapPoint);
void desktopFocus(DesktopWorld&, MapPoint);
void desktopHistory(DesktopWorld&, std::size_t);
