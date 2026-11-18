// Run the same city with the pinned reference and with the current core.
#include "lincity/lintypes.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"

#include <cstdlib>
#include <exception>
#include <filesystem>
#include <fstream>
#include <iostream>

#ifdef LINCITY_REFERENCE_TRACE
#include "lincity/modules/all_modules.hpp"
#include "lincity/resources.hpp"
#include "MainLincity.hpp"

static void initializeReferenceTrace() {
  initializeModules();
  simDelay = SIM_DELAY_FAST;
  const std::pair<const char*, int> animationLengths[] = {
#include "frame_counts.inc"
  };
  for(const auto& [name, count] : animationLengths) {
    const auto entry = ResourceGroup::resMap.find(name);
    if(entry != ResourceGroup::resMap.end())
      entry->second->graphicsInfoVector.resize(count);
  }
}
#endif

int main(int argc, char** argv) {
  if(argc != 3) return 2;
  try {
#ifdef LINCITY_REFERENCE_TRACE
    initializeReferenceTrace();
#endif
    std::srand(714);
    BasicUrbg::get().seed(714);
    auto world = World::load(argv[1]);
#ifndef LINCITY_REFERENCE_TRACE
    world->vehicleTraffic = World::VehicleTraffic::Disabled;
#endif
    const std::filesystem::path output(argv[2]);
    std::ofstream messages(output / "messages.txt");
    messages.exceptions(std::ios::badbit | std::ios::failbit);

    world->save(output / "day-0.gz");
    for(int day = 1; day <= NUMOF_DAYS_IN_YEAR + 1; ++day) {
      world->do_animate(1000UL * day);
      world->do_time_step();
      while(auto message = world->popMessage())
        messages << day << ' ' << message->str() << '\n';
      if(day == 1 || day == NUMOF_DAYS_IN_MONTH || day == NUMOF_DAYS_IN_YEAR + 1)
        world->save(output / ("day-" + std::to_string(day) + ".gz"));
    }
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
