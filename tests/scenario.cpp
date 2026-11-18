#include "lincity/lintypes.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"

#include <cstdlib>
#include <filesystem>
#include <iostream>
#include <stdexcept>

namespace {
void require(bool condition, const char* description) {
  if (!condition)
    throw std::runtime_error(description);
}

void advance(World& world, int days) {
  for (int day = 0; day < days; ++day) {
    world.do_animate(1000UL * (day + 1));
    world.do_time_step();
    while (world.popMessage()) {}
  }
}

void run(const std::filesystem::path& scenario,
         const std::filesystem::path& saved) {
  std::srand(714);
  BasicUrbg::get().seed(714);
  auto world = World::load(scenario);
  const int initialDay = world->total_time;
  require(!world->map.constructions.empty(), "legacy constructions missing");
  advance(*world, NUMOF_DAYS_IN_YEAR + 1);
  require(world->total_time == initialDay + NUMOF_DAYS_IN_YEAR + 1,
          "time did not advance across year boundary");

  world->save(saved);
  auto loaded = World::load(saved);
  require(loaded->total_time == world->total_time, "saved day changed");
  require(loaded->map.len() == world->map.len(), "saved map size changed");
  require(loaded->total_money == world->total_money, "saved balance changed");
  require(loaded->tech_level == world->tech_level, "saved tech level changed");
  require(loaded->map.recentPoint == world->map.recentPoint,
          "saved map position changed");
  require(loaded->map.constructions.size() == world->map.constructions.size(),
          "saved construction count changed");
  advance(*loaded, 1);
  require(loaded->total_time == world->total_time + 1,
          "loaded simulation did not advance");
}
}

int main(int argc, char** argv) {
  if (argc != 3)
    return 2;
  try {
    run(argv[1], argv[2]);
    std::filesystem::remove(argv[2]);
    return 0;
  } catch (const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
