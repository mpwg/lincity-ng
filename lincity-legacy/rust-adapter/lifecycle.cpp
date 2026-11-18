// Cross-language lifetime and simulation contract of the removable desktop adapter.
#include "DesktopWorld.hpp"
#include "lincity/modules/track_road_rail.hpp"
#include <cstdlib>
#include <iostream>

int main(int argc, char** argv) {
  if(argc != 3) return EXIT_FAILURE;
  try {
    auto a = DesktopWorld::load(argv[1]);
    auto b = DesktopWorld::load(argv[1]);
    const int day = a->total_time;
    const int money = a->total_money;
    const MapPoint site(3, 3);
    UserOperation build;
    build.action = UserOperation::ACTION_BUILD;
    build.constructionGroup = &trackConstructionGroup;
    UserOperation flood;
    flood.action = UserOperation::ACTION_FLOOD;
    if(desktopCost(*a, flood) != flood.cost(*a, {})) {
      std::cerr << "Toolbar flood price differs from the legacy display" << '\n';
      return EXIT_FAILURE;
    }
    Message::ptr reason;
    if(!desktopPreview(*a, build, site, reason)) return EXIT_FAILURE;
    desktopApply(*a, build, site);
    if(!a->map(site)->getConstruction() || a->total_money >= money) return EXIT_FAILURE;
    a->do_animate(1000);
    a->do_time_step();
    if(a->total_time != day + 1 || b->total_time != day || b->total_money != money)
      return EXIT_FAILURE;
    a->save(argv[2]);
    a.reset();
    auto resumed = DesktopWorld::load(argv[2]);
    if(resumed->total_time != day + 1 || !resumed->map(site)->getConstruction())
      return EXIT_FAILURE;
    UserOperation demolish;
    demolish.action = UserOperation::ACTION_BULLDOZE;
    desktopApply(*resumed, demolish, site);
    if(resumed->map(site)->getConstruction()) return EXIT_FAILURE;
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return EXIT_FAILURE;
  }
}
