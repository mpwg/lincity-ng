#include "lincity/modules/cricket.hpp"
#include "lincity/modules/firestation.hpp"
#include "lincity/modules/health_centre.hpp"
#include "lincity/modules/market.hpp"
#include "lincity/modules/monument.hpp"
#include "lincity/modules/port.hpp"
#include "lincity/modules/rocket_pad.hpp"
#include "lincity/modules/tip.hpp"
#include "lincity/world.hpp"

#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <system_error>

namespace {
namespace fs = std::filesystem;

#define CHECK(expression) do { if(!(expression)) throw std::runtime_error( \
  std::string(__func__) + " line " + std::to_string(__LINE__) + ": " #expression); } while(false)

template<typename Building>
const Building& buildingAt(const World& world, MapPoint point) {
  const auto* building = dynamic_cast<const Building*>(world.map(point)->getConstruction());
  CHECK(building);
  return *building;
}

void marketDirections(const fs::path& saved) {
  World world(32);
  const MapPoint point(12, 12);
  marketConstructionGroup.placeItem(world, point);
  CHECK(world.setMarketTrade(point, STUFF_FOOD, true, false));
  CHECK(world.setMarketTrade(point, STUFF_GOODS, false, true));
  world.save(saved);

  auto restored = World::load(saved);
  const auto& rules = buildingAt<Market>(*restored, point).commodityRuleCount;
  CHECK(rules[STUFF_FOOD].take && !rules[STUFF_FOOD].give);
  CHECK(!rules[STUFF_GOODS].take && rules[STUFF_GOODS].give);
}

void checkMigratedWorld(const World& world) {
  CHECK(world.given_scene == "Legacy accounting and history" && world.total_time == 309);
  CHECK(world.map.len() == 32 && world.map.recentPoint == MapPoint(12, 12));
  CHECK(world.tech_level == 7500 && world.stats.tech_level == 7500);
  // 467 at 15%, 105 at 20%, and an already-denominated windmill charge.
  CHECK(world.total_money == 1596);
  CHECK(world.stats.expenses.import.stat == 58 && world.stats.expenses.import.acc == 70);
  CHECK(world.stats.expenses.unemployment.acc == 21 && world.stats.expenses.windmill.acc == 13);
  CHECK(world.stats.income.income_tax.stat == 37 && world.taxable.labor == 91);
  CHECK(world.stats.population.unemployed_days_t == 2417);
  CHECK(!world.tradeRule[STUFF_FOOD].take && world.tradeRule[STUFF_FOOD].give);
  CHECK(world.tradeRule[STUFF_GOODS].take && !world.tradeRule[STUFF_GOODS].give);

  const auto& history = world.stats.history;
  CHECK(history.pop.at(0) == 901 && history.pop.at(2) == 701);
  CHECK(history.starve.at(1) == 23 && history.nojobs.at(2) == 27 && history.ppool.at(1) == 109);
  CHECK(history.tech.at(0) == 7500 && history.tech.at(1) == 7250);
  CHECK(history.tech.at(2) == 7000 && history.tech.at(3) == 7450);
  const auto& food = history.inventory[STUFF_FOOD];
  CHECK(food.at(0).amount == 660 && food.at(0).capacity == 1000);
  CHECK(food.at(1).amount == 620 && food.at(1).capacity == 1000);
  CHECK(history.tenants.at(0).amount == 910 && history.tenants.at(0).capacity == 1000);
  CHECK(history.tenants.at(1).amount == 890 && history.tenants.at(1).capacity == 1000);

  const auto& tile = *world.map(MapPoint(6, 8));
  CHECK(tile.group == GROUP_WATER && tile.is_river() && tile.ground.altitude == 17);
  CHECK(tile.flags & FLAG_HAS_UNDERGROUND_WATER);
  const auto& monument = buildingAt<Monument>(world, MapPoint(12, 12));
  CHECK(monument.tech_made == 3 && monument.tail_off == 2);
  CHECK(monument.completion == 18 && monument.labor_consumed == 63000);
}

void migrate2130(const fs::path& fixtures, const fs::path& saved) {
  auto world = World::load(fixtures / "migration-2130.xml");
  checkMigratedWorld(*world);
  world->save(saved);
  checkMigratedWorld(*World::load(saved));
}

const MapPoint readyRocket(13, 5);

void checkSpecializedBuildings(const World& world) {
  CHECK(world.map.constructionCount() == 8);
  const auto& building = buildingAt<RocketPad>(world, MapPoint(5, 5));
  CHECK(building.stage == RocketPad::BUILDING && building.steps == 450 && building.tech == 82000);
  CHECK(building.commodityCount[STUFF_LABOR] == 300);
  const auto& ready = buildingAt<RocketPad>(world, readyRocket);
  CHECK(ready.stage == RocketPad::AWAITING && ready.steps == 5000 && ready.tech == 78000);
  const auto& done = buildingAt<RocketPad>(world, MapPoint(21, 5));
  CHECK(done.stage == RocketPad::DONE && done.steps == 5000 && done.tech == 70000);

  const auto& port = buildingAt<Port>(world, MapPoint(12, 14));
  CHECK(port.tech_made == 9 && port.commodityCount[STUFF_GOODS] == 700);
  CHECK(port.commodityRuleCount[STUFF_FOOD].take && !port.commodityRuleCount[STUFF_FOOD].give);
  CHECK(!port.commodityRuleCount[STUFF_GOODS].take && port.commodityRuleCount[STUFF_GOODS].give);
  const auto& tip = buildingAt<Tip>(world, MapPoint(22, 13));
  CHECK(tip.total_waste == 14000 && tip.degration_days == 4);

  const auto& health = buildingAt<HealthCentre>(world, MapPoint(5, 22));
  CHECK(health.active && health.daycount == 8 && health.covercount == 31);
  const auto& firestation = buildingAt<FireStation>(world, MapPoint(13, 22));
  CHECK(!firestation.active && firestation.daycount == 4 && firestation.covercount == 29);
  const auto& cricket = buildingAt<Cricket>(world, MapPoint(21, 22));
  CHECK(cricket.active && cricket.daycount == 14 && cricket.covercount == 27);
}

void specializedBuildings(const fs::path& fixtures, const fs::path& saved) {
  auto world = World::load(fixtures / "specialized-buildings-2140.xml");
  checkSpecializedBuildings(*world);
  // Saving during a launch preserves the ready rocket so the player can retry.
  CHECK(world->launchRocket(readyRocket));
  CHECK(buildingAt<RocketPad>(*world, readyRocket).stage == RocketPad::LAUNCHING);
  world->save(saved);
  auto restored = World::load(saved);
  checkSpecializedBuildings(*restored);
  CHECK(restored->launchRocket(readyRocket));
}

void writeXml(const fs::path& path, std::string_view xml) {
  std::ofstream output(path, std::ios::binary | std::ios::trunc);
  output.exceptions(std::ios::failbit | std::ios::badbit);
  output << xml;
  output.close();
}

template<typename Operation>
void expectFailure(Operation operation, std::string_view diagnostic) {
  try {
    operation();
  } catch(const std::runtime_error& error) {
    if(std::string_view(error.what()).find(diagnostic) != std::string_view::npos) return;
    throw;
  }
  throw std::runtime_error("Expected rejection: " + std::string(diagnostic));
}

void parserErrors(const fs::path& saved) {
  writeXml(saved, "<SaveGame/>");
  expectFailure([&] { World::load(saved); }, "format that is too old");
  // A regular file cannot be the parent of a save path.
  const auto unavailable = saved / "unavailable.sav.gz";
  expectFailure([&] { World::load(unavailable); }, "failed to open file");
  const World world(32);
  expectFailure([&] { world.save(unavailable); }, "failed to open file");

  writeXml(saved, "<lc-game ldsv-version='2140'><map size='32'>"
                  "<recentPoint><y>7</y></recentPoint></map></lc-game>");
  expectFailure([&] { World::load(saved); }, "missing XML element <x>");
  writeXml(saved, "<lc-game ldsv-version='2140'><map size='32'>"
                  "<recentPoint><x>7</x></recentPoint></map></lc-game>");
  expectFailure([&] { World::load(saved); }, "missing XML element <y>");
  writeXml(saved, "<lc-game ldsv-version='2140'><map size='medium'/></lc-game>");
  expectFailure([&] { World::load(saved); }, "failed to parse XML value");
  writeXml(saved, "<lc-game ldsv-version='2140'><globals><game_end>maybe</game_end>"
                  "</globals></lc-game>");
  expectFailure([&] { World::load(saved); }, "failed to parse bool");

  for(const char* legacyField : {
    "<monthgraph_pop>90 missing 70</monthgraph_pop>",
    "<pbar id='1'><data>7000 missing</data></pbar>",
    "<pbar id='3'><data>660 missing</data></pbar>"
  }) {
    writeXml(saved, std::string("<lc-game ldsv-version='2130'><globals>") + legacyField
                   + "</globals></lc-game>");
    try {
      World::load(saved);
    } catch(const std::system_error& error) {
      CHECK(error.code() == std::errc::invalid_argument);
      continue;
    }
    throw std::runtime_error(std::string("Expected rejection of legacy field: ") + legacyField);
  }
}

#ifdef NDEBUG
void unknownFields(const fs::path& saved) {
  // Release builds skip unknown fields and keep reading the known siblings.
  writeXml(saved, R"xml(
    <lc-game ldsv-version="2140">
      <extension/>
      <globals>
        <stats>
          <history>
            <money>
              <metadata>123</metadata>
              <index_latest>43</index_latest>
              <index_99>99</index_99>
              <index_1>-37</index_1>
            </money>
            <food>
              <index_0><label>Winter</label><amount>45</amount><capacity>100</capacity></index_0>
            </food>
          </history>
        </stats>
        <total_money>81</total_money>
      </globals>
      <map size="32">
        <extension/>
        <recentPoint><x>6</x><zoom>2</zoom><y>7</y></recentPoint>
      </map>
    </lc-game>
  )xml");
  auto world = World::load(saved);
  CHECK(world->total_money == 81 && world->map.recentPoint == MapPoint(6, 7));
  CHECK(world->stats.history.money.at(0) == 0 && world->stats.history.money.at(1) == -37);
  const auto& food = world->stats.history.inventory[STUFF_FOOD].at(0);
  CHECK(food.amount == 45 && food.capacity == 100);
}
#endif
}

int main(int argc, char** argv) {
  if(argc != 3) return 2;
  const fs::path fixtures = argv[1];
  const fs::path saved = argv[2];
  try {
    marketDirections(saved);
    migrate2130(fixtures, saved);
    specializedBuildings(fixtures, saved);
    parserErrors(saved);
#ifdef NDEBUG
    unknownFields(saved);
#endif
    fs::remove(saved);
  } catch(const std::exception& ex) {
    std::cerr << ex.what() << '\n';
    return 1;
  }
}
