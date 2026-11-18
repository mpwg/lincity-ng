#include "lincity/ConstructionReport.hpp"
#include "lincity/lintypes.hpp"
#include "lincity/modules/blacksmith.hpp"
#include "lincity/modules/mill.hpp"
#include "lincity/modules/residence.hpp"
#include "lincity/modules/solar_power.hpp"
#include "lincity/modules/power_line.hpp"
#include "lincity/modules/substation.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"

#include <cstdlib>
#include <iostream>
#include <stdexcept>
#include <tuple>
#include <vector>

namespace {
class Report : public ConstructionReport {
public:
  int rows = 0;
  using Quantity = std::tuple<std::string, std::string, int, int>;
  std::vector<Quantity> quantities;

  void addBlank() override { ++rows; }
  void addLabel(const std::string&) override { ++rows; }
  void addLabeledText(const std::string&, const std::string&) override { ++rows; }
  void addInteger(const std::string&, int) override { ++rows; }
  void addDecimal(const std::string&, double) override { ++rows; }
  void addPercentage(const std::string&, double) override { ++rows; }
  void addQuantity(const std::string&, int, int) override { ++rows; }
  void addPrefixedQuantity(const std::string& prefix, const std::string& label,
                           int value, int capacity) override {
    quantities.emplace_back(prefix, label, value, capacity);
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
  auto& building = *world.map(site)->getConstruction();
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
  auto& building = *world.map(site)->getConstruction();
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
  auto& building = *world.map(site)->getConstruction();
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

void verifyInventoryAndProductionReport() {
  World world(32);
  const MapPoint site(12, 12);
  blacksmithConstructionGroup.placeItem(world, site);
  auto& building = *world.map(site)->getConstruction();
  building.commodityCount[STUFF_COAL] = BLACKSMITH_COAL_USED;
  building.commodityCount[STUFF_STEEL] = BLACKSMITH_STEEL_USED;
  building.commodityCount[STUFF_LABOR] = BLACKSMITH_LABOR;
  world.total_time = 99;
  building.update();

  Report production;
  building.report(production, true);
  const std::vector<Report::Quantity> expectedProduction{
    {"<-- ", "Goods", GOODS_MADE_BY_BLACKSMITH, building.commodityMaxProd[STUFF_GOODS]},
    {"--> ", "Labor", -BLACKSMITH_LABOR, -building.commodityMaxCons[STUFF_LABOR]},
    {"--> ", "Coal", -BLACKSMITH_COAL_USED, -building.commodityMaxCons[STUFF_COAL]},
    {"--> ", "Steel", -BLACKSMITH_STEEL_USED, -building.commodityMaxCons[STUFF_STEEL]}
  };
  require(production.quantities == expectedProduction, "wrong monthly production rows");

  world.evacuateArea(site);
  Report inventory;
  building.report(inventory, false);
  const std::vector<Report::Quantity> expectedInventory{
    {"<<< ", "Goods", GOODS_MADE_BY_BLACKSMITH, MAX_GOODS_AT_BLACKSMITH},
    {"<<< ", "Labor", 0, MAX_LABOR_AT_BLACKSMITH},
    {"<<< ", "Coal", 0, MAX_COAL_AT_BLACKSMITH},
    {"<<< ", "Steel", 0, MAX_STEEL_AT_BLACKSMITH}
  };
  require(inventory.quantities == expectedInventory, "wrong evacuating inventory rows");
  require(Construction::getStuffName(STUFF_COAL) == "Coal",
          "invalid commodity display name in the finance report");
}

void verifyResidenceFootprints() {
  for(auto* group : {&residenceLLConstructionGroup, &residenceMLConstructionGroup,
                    &residenceHLConstructionGroup, &residenceLHConstructionGroup,
                    &residenceMHConstructionGroup, &residenceHHConstructionGroup}) {
    World world(32);
    const MapPoint site(12, 12);
    group->placeItem(world, site);
    const MapPoint corner = site.e(group->size - 1).s(group->size - 1);
    require(world.map(site)->is_residence() && world.map(corner)->is_residence(),
            "residential footprint is not recognized");
    require(!world.map(site.w())->is_residence(), "a bare tile reported a residence");
    world.bulldozeArea(corner);
    require(!world.map(site)->is_residence() && !world.map(corner)->is_residence(),
            "removed residence kept part of its footprint");
    world.do_time_step();
    require(world.map.constructionCount() == 0, "detached residence was not pruned");
  }
}

void verifyDetachedConstructionLifetime() {
  struct ObservedSmithy : Blacksmith {
    ObservedSmithy(World& world, int& destructions)
      : Blacksmith(world, &blacksmithConstructionGroup), destructions(destructions) {}
    ~ObservedSmithy() override { ++destructions; }

    int& destructions;
  };

  int destructions = 0;
  {
    World world(32);
    const MapPoint site(12, 12);
    (new ObservedSmithy(world, destructions))->place(site);
    world.bulldozeArea(site);
    require(destructions == 0, "detached building deleted before the simulation boundary");
    (new ObservedSmithy(world, destructions))->place(site);
  }
  require(destructions == 2, "ending the city lost a detached construction");
}

void verifyPowerCableLimits() {
  World world(32);
  const MapPoint site(12, 12);
  const int size = substationConstructionGroup.size;
  substationConstructionGroup.placeItem(world, site);
  struct Cable { MapPoint water; MapPoint pole; int mask; unsigned flag; };
  const Cable cables[] = {
    {site.n(), site.n(2), 8, FLAG_POWER_CABLES_0},
    {site.w(), site.w(2), 4, FLAG_POWER_CABLES_90},
    {site.e(size), site.e(size + 1), 2, FLAG_POWER_CABLES_90},
    {site.s(size), site.s(size + 1), 1, FLAG_POWER_CABLES_0},
  };
  for(const auto& link : cables) {
    world.map(link.water)->setTerrain(GROUP_WATER);
    powerlineConstructionGroup.placeItem(world, link.pole);
  }

  world.map(site.ne())->setTerrain(GROUP_WATER);
  powerlineConstructionGroup.placeItem(world, site.n(2).e());
  world.map.connect_transport(8, 8, 18, 18);
  auto& station = *world.map(site)->getConstruction();
  for(const auto& link : cables) {
    require(station.countPowercables(link.mask) == 1,
            "power line did not link to the intended side");
    require(world.map(link.water)->flags & link.flag,
            "power line did not span the intended water tile");
  }
  require(!(world.map(site.ne())->flags & FLAG_POWER_CABLES_0),
          "two suspended cables were attached on the same side");

  world.bulldozeArea(site.n(2));
  require(station.countPowercables(8) == 1, "remaining north cable did not take over");
  require(world.map(site.ne())->flags & FLAG_POWER_CABLES_0, "north cable did not relocate");
  require(!(world.map(site.n())->flags & FLAG_POWER_CABLES_0), "removed north cable survived");
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
  auto* building = world.map(origin)->getConstruction();
  require(building && building->getConstructionGroup() == &group, "wrong building placed");
  for(int y = 0; y < group.size; ++y) {
    for(int x = 0; x < group.size; ++x)
      require(world.map(origin.e(x).s(y))->getReportingConstruction() == building,
              "incomplete footprint");
  }
  for(unsigned long frame = 1; frame <= 3; ++frame) {
    world.do_animate(frame * 1000);
    world.do_time_step();
  }
  require(world.total_time == 3, "world failed to tick");
  if(world.map(origin)->getConstruction() != building) return;
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
    verifyInventoryAndProductionReport();
    verifyResidenceFootprints();
    verifyDetachedConstructionLifetime();
    verifyPowerCableLimits();
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
