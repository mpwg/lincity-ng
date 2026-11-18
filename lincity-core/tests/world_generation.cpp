#include "lincity/groups.hpp"
#include "lincity/init_game.hpp"
#include "util/randutil.hpp"

#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <stdexcept>

namespace {
using Generator = std::unique_ptr<World> (*)(city_settings*, int);

struct Reference {
  const char* name;
  Generator generate;
  std::uint64_t wooded, cleared;
  int trees;
};

// v1, seed 615, size 100. Captured from the e737f664 Linux x86-64
// GCC/libstdc++/glibc release build before the simulation trace was added.
constexpr Reference maps[] = {
  {"river", new_city, 0xb6516824caf8946d, 0xbf6f8ee2536702b2, 6347},
  {"desert", new_desert_city, 0xab2ea678d424c464, 0xe9cd2c63acc03001, 1022},
  {"temperate", new_temperate_city, 0x9208bc16db745cef, 0x0379e5e4fafc3f28, 2704},
  {"swamp", new_swamp_city, 0x0830ac3033541d7d, 0xfa60f90a549471c6, 7114},
};

void require(bool condition, const char* description) {
  if(!condition) throw std::runtime_error(description);
}

std::unique_ptr<World> generate(Generator generator, city_settings settings) {
  std::srand(615);
  BasicUrbg::get().seed(615);
  return generator(&settings, 100);
}

// FNV-1a over row-major terrain, with each field encoded as a little-endian
// 32-bit word so the reference can also be computed outside C++.
std::uint64_t terrainHash(const Map& map) {
  std::uint64_t hash = UINT64_C(14695981039346656037);
  for(const auto& tile : map) {
    const std::uint32_t fields[] = {
      tile.group, tile.type, tile.flags, std::uint32_t(tile.ground.altitude),
      std::uint32_t(tile.ground.ecotable), std::uint32_t(tile.ground.water_alt),
      tile.coal_reserve, tile.ore_reserve
    };
    for(auto value : fields)
      for(int i = 0; i < 4; ++i) {
        hash ^= value & 0xff;
        hash *= UINT64_C(1099511628211);
        value >>= 8;
      }
  }
  return hash;
}

void verify(const Reference& ref) {
  for(bool withoutTrees : {false, true}) {
    auto world = generate(ref.generate, {false, withoutTrees});
    require(world->map.len() == 100, "generated map size differs");
    require(world->map.constructionCount() == 0, "village built when disabled");
    const auto trees = std::count_if(world->map.begin(), world->map.end(),
      [](const MapTile& tile) {
        return tile.group == GROUP_TREE || tile.group == GROUP_TREE2 ||
          tile.group == GROUP_TREE3;
      });
    require(withoutTrees ? trees == 0 : trees > 0, "tree setting was ignored");
    auto hash = terrainHash(world->map);
    // rand(), distributions, floating point and argument evaluation order
    // belong to the native toolchain. Other platforms retain local replay.
#if defined(__GLIBC__) && defined(__GLIBCXX__) && defined(__GNUC__) && !defined(__clang__) && defined(__x86_64__)
    const auto expected = withoutTrees ? ref.cleared : ref.wooded;
    if(hash != expected) {
      std::cerr << "withoutTrees=" << withoutTrees << " expected=" << std::hex
                << expected << " observed=" << hash << std::dec << '\n';
      throw std::runtime_error("reference terrain differs");
    }
    require(trees == (withoutTrees ? 0 : ref.trees), "reference tree count differs");
#endif
    auto replay = generate(ref.generate, {false, withoutTrees});
    require(terrainHash(replay->map) == hash, "seeded terrain did not replay");
  }
  auto village = generate(ref.generate, {true, true});
  require(village->map.constructionCount() > 0, "starter village missing");
  village->do_time_step();
  require(village->total_time == 1, "new village did not simulate");
}
}

int main() {
  for(const auto& ref : maps) {
    try {
      verify(ref);
    } catch(const std::exception& error) {
      std::cerr << ref.name << ": " << error.what() << '\n';
      return 1;
    }
  }
}
