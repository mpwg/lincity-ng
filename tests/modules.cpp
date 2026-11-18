#include "lincity/ConstructionReport.hpp"
#include "lincity/lintypes.hpp"
#include "lincity/modules/blacksmith.hpp"
#include "lincity/modules/mill.hpp"
#include "lincity/modules/solar_power.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"

#include <cstdlib>
#include <iostream>
#include <stdexcept>

namespace {
class Report : public ConstructionReport {
public:
  int rows = 0;

  void addBlank() override { ++rows; }
  void addLabel(const std::string&) override { ++rows; }
  void addLabeledText(const std::string&, const std::string&) override { ++rows; }
  void addInteger(const std::string&, int) override { ++rows; }
  void addDecimal(const std::string&, double) override { ++rows; }
  void addPercentage(const std::string&, double) override { ++rows; }
  void addQuantity(const std::string&, int, int) override { ++rows; }
  void addPrefixedQuantity(const std::string&, const std::string&, int, int) override {
    ++rows;
  }
};

void require(bool condition, const char* description) {
  if(!condition) throw std::runtime_error(description);
}

void verifyBlacksmith() {
  World world(32);
  const MapPoint site(12, 12);
  blacksmithConstructionGroup.placeItem(world, site);
  auto& building = *world.map(site)->construction;
  building.levelStuff(STUFF_COAL, BLACKSMITH_COAL_USED);
  building.levelStuff(STUFF_STEEL, BLACKSMITH_STEEL_USED);
  building.levelStuff(STUFF_LABOR, BLACKSMITH_LABOR - 1);
  building.update();
  require(building.commodityCount[STUFF_GOODS] == 0, "blacksmith produced without labor");
  require(building.commodityCount[STUFF_STEEL] == BLACKSMITH_STEEL_USED,
          "idle blacksmith consumed steel");
  building.levelStuff(STUFF_LABOR, BLACKSMITH_LABOR);
  building.update();
  require(building.commodityCount[STUFF_GOODS] == GOODS_MADE_BY_BLACKSMITH,
          "blacksmith produced the wrong amount");
  for(auto input : {STUFF_STEEL, STUFF_COAL, STUFF_LABOR})
    require(building.commodityCount[input] == 0, "blacksmith did not consume an input");
}

void verifyMill(Commodity fuel, int amount) {
  World world(32);
  const MapPoint site(12, 12);
  millConstructionGroup.placeItem(world, site);
  auto& building = *world.map(site)->construction;
  building.levelStuff(STUFF_FOOD, FOOD_USED_BY_MILL);
  building.levelStuff(STUFF_LABOR, MILL_LABOR);
  building.levelStuff(fuel, amount);
  building.update();
  require(building.commodityCount[STUFF_GOODS] == GOODS_MADE_BY_MILL,
          "mill produced the wrong amount");
  require(building.commodityCount[fuel] == 0, "mill did not consume its fuel");
}

void verifySolarPower(int tech, int output) {
  World world(32);
  world.tech_level = tech;
  const MapPoint site(12, 12);
  solarPowerConstructionGroup.placeItem(world, site);
  auto& building = *world.map(site)->construction;
  building.levelStuff(STUFF_LABOR, SOLAR_POWER_LABOR);
  building.update();
  require(building.commodityCount[STUFF_HIVOLT] == output, "solar output changed");
  building.levelStuff(STUFF_LABOR, SOLAR_POWER_LABOR);
  building.levelStuff(STUFF_HIVOLT, MAX_HIVOLT_AT_SOLARPS);
  building.update();
  require(building.commodityCount[STUFF_HIVOLT] == MAX_HIVOLT_AT_SOLARPS,
          "solar plant exceeded its capacity");
  require(building.commodityCount[STUFF_LABOR] == SOLAR_POWER_LABOR,
          "idle solar plant consumed labor");
}

void verifyModule(ConstructionGroup& group) {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  world.total_money = 1000000;
  const MapPoint origin(12, 12);
  if(group.group == GROUP_TRACK_BRIDGE || group.group == GROUP_ROAD_BRIDGE ||
     group.group == GROUP_RAIL_BRIDGE)
    world.map(origin)->setTerrain(GROUP_WATER);
  group.placeItem(world, origin);
  auto* building = world.map(origin)->construction;
  require(building && building->constructionGroup == &group, "wrong building placed");
  for(int y = 0; y < group.size; ++y) {
    for(int x = 0; x < group.size; ++x)
      require(world.map(origin.e(x).s(y))->reportingConstruction == building,
              "incomplete footprint");
  }
  for(unsigned long frame = 1; frame <= 3; ++frame) {
    world.do_animate(frame * 1000);
    world.do_time_step();
  }
  require(world.total_time == 3, "world failed to tick");
  if(world.map(origin)->construction != building) return;
  for(bool production : {false, true}) {
    Report report;
    building->report(report, production);
    require(report.rows > 0, "empty building report");
  }
}
}

int main() {
  std::srand(724);
  BasicUrbg::get().seed(724);
  World registration(32);
  int modules = 0;
  for(unsigned short id = 0; id < NUM_OF_GROUPS; ++id) {
    auto* group = ConstructionGroup::getConstructionGroup(id);
    if(!group) continue;
    try {
      verifyModule(*group);
      ++modules;
    } catch(const std::exception& error) {
      std::cerr << group->name << ": " << error.what() << '\n';
      return 1;
    }
  }
  if(modules != 43) {
    std::cerr << "Expected 43 building modules, exercised " << modules << '\n';
    return 1;
  }
  try {
    verifyBlacksmith();
    verifyMill(STUFF_COAL, COAL_USED_BY_MILL);
    verifyMill(STUFF_LOVOLT, COAL_USED_BY_MILL * MILL_POWER_PER_COAL);
    verifySolarPower(0, POWERS_SOLAR_OUTPUT);
    verifySolarPower(MAX_TECH_LEVEL, 2 * POWERS_SOLAR_OUTPUT);
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
  std::cout << modules << " building modules exercised\n";
}
