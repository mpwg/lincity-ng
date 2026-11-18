#include "lincity/groups.hpp"
#include "lincity/init_game.hpp"
#include "lincity/lintypes.hpp"
#include "util/randutil.hpp"
#include "util/simulation_trace.hpp"

#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <limits>
#include <sstream>
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

std::unique_ptr<World> generate(Generator generator, city_settings settings,
                                unsigned seed = 615, int size = 100) {
  std::srand(seed);
  BasicUrbg::get().seed(seed);
  return generator(&settings, size);
}

void verifyInvalidDimensionsAndSettings() {
  const auto invalid = [](auto operation, const char* description) {
    try {
      operation();
    } catch(const std::invalid_argument&) {
      return;
    }
    throw std::runtime_error(description);
  };
  invalid([] { World world(2); }, "a world without visible tiles was accepted");
  invalid([] { World world(0); }, "a zero-size world was accepted");
  invalid([] { World world(-1); }, "a negative world size was accepted");
  invalid([] { World world(std::numeric_limits<int>::max()); },
          "a world with an unrepresentable area was accepted");

  World minimum(3);
  require(minimum.map.is_visible(MapPoint(1, 1)), "minimum world lost its visible tile");
  minimum.do_time_step();
  require(minimum.total_time == 1, "minimum world did not simulate");

  city_settings settings{false, true};
  invalid([&] { create_new_city(&settings, 50, false, 4); }, "invalid climate was accepted");
  invalid([&] { create_new_city(&settings, 50, false, -1); }, "negative climate was accepted");
  invalid([] { new_temperate_city(nullptr, 50); }, "missing city settings were accepted");
  invalid([&] { new_temperate_city(&settings, 12); }, "invalid terrain size was accepted");
  settings.with_village = true;
  invalid([&] { new_temperate_city(&settings, 25); }, "invalid village size was accepted");

  auto bare = generate(new_city, {false, true}, 615, 13);
  require(bare->map.len() == 13 && bare->map.constructionCount() == 0,
          "minimum terrain size did not generate an empty city");
  auto village = generate(new_temperate_city, settings, 448, 26);
  require(village->map.len() == 26 && village->stats.groupCount[GROUP_MARKET] == 1,
          "minimum village size did not generate a market");
}

void verifyLakeDensity() {
  require(terrainLakeCount(0, 10, 100) == 10, "ordinary map has the wrong lake count");
  require(terrainLakeCount(200, 10, 50) == 1, "fractional lake count was not rounded down");
  require(terrainLakeCount(-200, 15, 10000) == 225000,
          "large wet map has the wrong lake count");
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

void verifyVillageReplay(const Reference& ref) {
  city_settings city{true, false};
  std::ostringstream tape;
  std::unique_ptr<World> original;
  {
    SimulationTrace recording(tape);
    original = generate(ref.generate, city);
    recording.finish();
  }

  std::srand(923);
  BasicUrbg::get().seed(317);
  std::istringstream input(tape.str());
  SimulationTrace replay(input);
  const auto rebuilt = ref.generate(&city, 100);
  replay.finish();
  require(original->map.constructionCount() > 0, "recorded village is empty");
  require(rebuilt->map.constructionCount() == original->map.constructionCount(),
          "village construction count differs in replay");
  require(terrainHash(original->map) == terrainHash(rebuilt->map), "village terrain differs in replay");
  for(const auto& tile : original->map) {
    const auto& copy = *rebuilt->map(tile.point);
    const auto* building = tile.getConstruction();
    const auto* restored = copy.getConstruction();
    require((building == nullptr) == (restored == nullptr), "village footprint differs in replay");
    if(building) {
      require(tile.getTopGroup() == copy.getTopGroup(), "village building differs in replay");
      require(building->commodityCount == restored->commodityCount, "village inventory differs in replay");
    }
  }
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

void verifySmallCoastalMap() {
  constexpr int minimumDesktopSize = 50;
  struct Coast {
    unsigned seed;
    int seaLevel;
    int shallowWaterTiles;
    bool limited;
  };
  const Coast examples[] = {
    {4, 1467, 397, false},
    {448, 4054, 1037, true},
  };
  for(const auto& coast : examples) {
    city_settings landscape{false, true};
    std::ostringstream tape;
    std::unique_ptr<World> world;
    {
      SimulationTrace recording(tape);
      world = generate(new_temperate_city, landscape, coast.seed, minimumDesktopSize);
      recording.finish();
    }
    require(world->map.len() == minimumDesktopSize && world->map.constructionCount() == 0,
            "coastal map did not use the requested size or included a disabled village");
    const MapPoint innerEdge(minimumDesktopSize - 2, minimumDesktopSize / 2);
    require(world->map.is_edge(innerEdge) && world->map.is_visible(innerEdge) &&
            !world->map.is_edge(innerEdge.w()) && world->map.is_border(innerEdge.e()) &&
            !world->map.is_visible(innerEdge.e()), "small map misplaced its visible edge and border");
    int lowest = std::numeric_limits<int>::max();
    int landTiles = 0;
    for(const auto& tile : world->map) {
      if(!world->map.is_visible(tile.point)) continue;
      lowest = std::min(lowest, tile.ground.altitude);
      if(!tile.is_water()) ++landTiles;
    }
    require(landTiles > 0 && world->map.alt_step > 0,
            "small coastal generator did not retain land or a usable altitude scale");

#if defined(__GLIBC__) && defined(__GLIBCXX__) && defined(__GNUC__) && !defined(__clang__) && defined(__x86_64__)
    require(lowest == coast.seaLevel, "unexpected minimum elevation on the small coastal map");
    int shallowWater = 0;
    for(const auto& tile : world->map) {
      if(!world->map.is_visible(tile.point) || tile.ground.altitude != lowest || !tile.is_water())
        continue;
      require(tile.is_river() && (tile.flags & FLAG_HAS_UNDERGROUND_WATER),
              "flooded coastal land was not connected surface water with groundwater");
      ++shallowWater;
    }
    require(shallowWater == coast.shallowWaterTiles, "unexpected area flooded at sea level");
    const int seaLimit = world->map.alt_max / 3;
    require(coast.limited ? lowest == seaLimit : lowest < seaLimit,
            "coast did not respect the sea level limit");
#endif

    std::istringstream input(tape.str());
    SimulationTrace replay(input);
    const auto restored = new_temperate_city(&landscape, minimumDesktopSize);
    replay.finish();
    require(terrainHash(restored->map) == terrainHash(world->map),
            "small coastal terrain changed during replay");
  }

  auto village = generate(new_temperate_city, {true, true}, 448, minimumDesktopSize);
  require(village->map.len() == minimumDesktopSize &&
          village->stats.groupCount[GROUP_MARKET] == 1 &&
          village->stats.groupCount[GROUP_RESIDENCE_ML] == 2,
          "small map did not contain a complete starter village");
  village->do_time_step();
  require(village->total_time == 1, "starter village on the smallest map did not simulate");
}
}

int main() {
  try {
    verifyInvalidDimensionsAndSettings();
    verifyLakeDensity();
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
  for(const auto& ref : maps) {
    try {
      verify(ref);
      verifyVillageReplay(ref);
    } catch(const std::exception& error) {
      std::cerr << ref.name << ": " << error.what() << '\n';
      return 1;
    }
  }
  try {
    verifySmallCoastalMap();
  } catch(const std::exception& error) {
    std::cerr << "small coast: " << error.what() << '\n';
    return 1;
  }
}
