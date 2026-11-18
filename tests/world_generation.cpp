#include "lincity/groups.hpp"
#include "lincity/init_game.hpp"
#include "util/randutil.hpp"

#include <algorithm>
#include <cstdlib>
#include <iostream>
#include <stdexcept>

namespace {
using Generator = std::unique_ptr<World> (*)(city_settings*, int);

void require(bool condition, const char* description) {
  if(!condition) throw std::runtime_error(description);
}

std::unique_ptr<World> generate(Generator generator, city_settings settings) {
  std::srand(615);
  BasicUrbg::get().seed(615);
  return generator(&settings, 100);
}

void verify(Generator generator) {
  auto first = generate(generator, {false, true});
  auto replay = generate(generator, {false, true});
  require(first->map.len() == 100, "generated map size differs");
  require(first->map.constructions.empty(), "village built when disabled");
  const auto sameTerrain = [](const MapTile& a, const MapTile& b) {
    return a.group == b.group && a.type == b.type && a.flags == b.flags &&
      a.ground.altitude == b.ground.altitude && a.coal_reserve == b.coal_reserve &&
      a.ore_reserve == b.ore_reserve;
  };
  require(std::equal(first->map.begin(), first->map.end(), replay->map.begin(),
          replay->map.end(), sameTerrain), "seeded terrain did not replay");
  require(std::none_of(first->map.begin(), first->map.end(),
          [](const MapTile& tile) {
            return tile.group == GROUP_TREE || tile.group == GROUP_TREE2 ||
              tile.group == GROUP_TREE3;
          }), "trees generated with trees disabled");
  auto village = generate(generator, {true, true});
  require(!village->map.constructions.empty(), "starter village missing");
  village->do_time_step();
  require(village->total_time == 1, "new village did not simulate");
}
}

int main() {
  for(auto generator : {new_city, new_desert_city, new_temperate_city, new_swamp_city}) {
    try {
      verify(generator);
    } catch(const std::exception& error) {
      std::cerr << error.what() << '\n';
      return 1;
    }
  }
}
