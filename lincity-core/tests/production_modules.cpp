#include "lincity/modules/coal_power.hpp"
#include "lincity/modules/coalmine.hpp"
#include "lincity/modules/commune.hpp"
#include "lincity/modules/heavy_industry.hpp"
#include "lincity/modules/light_industry.hpp"
#include "lincity/modules/mill.hpp"
#include "lincity/modules/oremine.hpp"
#include "lincity/modules/organic_farm.hpp"
#include "lincity/modules/pottery.hpp"
#include "lincity/modules/power_line.hpp"
#include "lincity/modules/recycle.hpp"
#include "lincity/modules/solar_power.hpp"
#include "lincity/modules/windmill.hpp"
#include "lincity/modules/windpower.hpp"
#include "lincity/world.hpp"

#include <initializer_list>
#include <iostream>
#include <stdexcept>
#include <string>
#include <utility>

namespace {
const MapPoint site(12, 12);

#define CHECK(expression) do { if(!(expression)) throw std::runtime_error( \
  std::string(__func__) + " line " + std::to_string(__LINE__) + ": " #expression); } while(false)

Construction& place(World& world, ConstructionGroup& group) {
  group.placeItem(world, site);
  return *world.map(site)->getConstruction();
}

void stock(Construction& building, std::initializer_list<std::pair<Commodity, int>> amounts) {
  // Delivery changes inventory without counting as local production.
  for(auto [commodity, amount] : amounts) building.commodityCount[commodity] = amount;
}

void coalPower() {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  auto& plant = place(world, coal_powerConstructionGroup);
  auto& stored = plant.commodityCount;
  stock(plant, {{STUFF_LABOR, 100}, {STUFF_COAL, 43}});
  plant.update();
  CHECK(stored[STUFF_HIVOLT] == 0 && stored[STUFF_LABOR] == 100);
  CHECK(world.map(site)->pollution == 0);

  stock(plant, {{STUFF_COAL, 44}});
  plant.update();
  CHECK(stored[STUFF_HIVOLT] == 22000 && stored[STUFF_LABOR] == 0 && stored[STUFF_COAL] == 0);
  CHECK(world.map(site)->pollution == 20);

  stock(plant, {{STUFF_HIVOLT, 209001}, {STUFF_LABOR, 50}, {STUFF_COAL, 22}});
  plant.update();
  CHECK(stored[STUFF_HIVOLT] == 209001 && stored[STUFF_LABOR] == 50 && stored[STUFF_COAL] == 22);
  stock(plant, {{STUFF_HIVOLT, 209000}, {STUFF_LABOR, 49}});
  plant.update();
  CHECK(stored[STUFF_HIVOLT] == 209000 && stored[STUFF_COAL] == 22);
  stock(plant, {{STUFF_LABOR, 50}});
  world.total_time = 99;
  plant.update();
  CHECK(stored[STUFF_HIVOLT] == 220000 && stored[STUFF_COAL] == 0 && stored[STUFF_LABOR] == 0);
  CHECK(world.map(site)->pollution == 30);
  CHECK(plant.commodityProdPrev[STUFF_HIVOLT] == 33000);
}

void coalMineThresholds() {
  World world(32);
  auto& mine = place(world, coalmineConstructionGroup);
  auto& stored = mine.commodityCount;
  world.stats.sustainability.mining_flag = true;
  CHECK(world.map(site)->coal_reserve == 20);

  stock(mine, {{STUFF_COAL, 15000}, {STUFF_LABOR, 917}});
  mine.update();
  CHECK(stored[STUFF_COAL] == 15000 && world.map(site)->coal_reserve == 20);
  stock(mine, {{STUFF_LABOR, 918}, {STUFF_COAL, 15001}});
  mine.update();
  CHECK(stored[STUFF_COAL] == 15001 && stored[STUFF_LABOR] == 918);
  stock(mine, {{STUFF_COAL, 15000}});
  mine.update();
  CHECK(stored[STUFF_COAL] == 16000 && stored[STUFF_LABOR] == 0);
  CHECK(world.map(site)->coal_reserve == 19 && world.map(site)->pollution == 3);
  CHECK(!world.stats.sustainability.mining_flag);

  stock(mine, {{STUFF_COAL, 16999}, {STUFF_LABOR, 918}});
  mine.update();
  CHECK(stored[STUFF_COAL] == 16999 && stored[STUFF_LABOR] == 918);
  stock(mine, {{STUFF_COAL, 17000}});
  mine.update();
  CHECK(stored[STUFF_COAL] == 16000 && stored[STUFF_LABOR] == 0);
  CHECK(world.map(site)->coal_reserve == 20 && world.map(site)->pollution == 3);
}

void coalMineFullDeposit() {
  for(int lastReserve : {9999, 10000}) {
    World world(32);
    for(int y = 6; y < 18; ++y)
      for(int x = 6; x < 18; ++x)
        world.map(MapPoint(x, y))->coal_reserve = 10000;
    auto& lastDeposit = *world.map(site.e(5).s(5));
    lastDeposit.coal_reserve = lastReserve;
    auto& mine = place(world, coalmineConstructionGroup);
    stock(mine, {{STUFF_COAL, 17000}, {STUFF_LABOR, 918}});
    mine.update();
    if(lastReserve == 9999) {
      CHECK(mine.commodityCount[STUFF_COAL] == 16000 && mine.commodityCount[STUFF_LABOR] == 0);
    } else {
      CHECK(mine.commodityCount[STUFF_COAL] == 17000 && mine.commodityCount[STUFF_LABOR] == 918);
    }
    CHECK(lastDeposit.coal_reserve == 10000);
    CHECK(world.map(site.n(6).w(6))->coal_reserve == 10000);
  }
}

void coalMineExhaustion() {
  World world(32);
  auto& mine = place(world, coalmineConstructionGroup);
  // A competing mine has removed the deposit since this mine was placed.
  world.map(site)->coal_reserve = 0;
  stock(mine, {{STUFF_LABOR, 918}});
  mine.update();
  CHECK(mine.flags & FLAG_EVACUATE);
  CHECK(world.map(site)->getConstruction() == &mine && mine.commodityCount[STUFF_LABOR] == 918);
  stock(mine, {{STUFF_LABOR, 0}});
  mine.update();
  CHECK(!world.map(site)->getConstruction());
  world.do_time_step(); // The simulation reclaims the detached construction on the next tick.
}

void oreMineConstruction() {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  world.total_money = 1000000;
  world.map(site)->ore_reserve = 499;
  world.map(site.e())->ore_reserve = 500;
  Message::ptr message;
  CHECK(!oremineConstructionGroup.can_build_here(world, site, message));
  CHECK(dynamic_message_cast<NoOreMessage>(message));
  world.map(site.s())->ore_reserve = 1;
  CHECK(oremineConstructionGroup.can_build_here(world, site, message));

  auto& mine = place(world, oremineConstructionGroup);
  mine.bulldoze();
  CHECK(world.map(site)->is_water());
  CHECK(!world.map(site.e())->is_water());
  CHECK(world.map(site.s())->is_water());
  world.do_time_step();
}

void oreMineThresholds() {
  World world(32);
  world.map(site.e())->ore_reserve = 1000;
  auto& mine = place(world, oremineConstructionGroup);
  auto& stored = mine.commodityCount;
  world.stats.sustainability.mining_flag = true;

  stock(mine, {{STUFF_ORE, 76000}, {STUFF_LABOR, 208}});
  mine.update();
  CHECK(stored[STUFF_ORE] == 76000 && world.map(site.e())->ore_reserve == 1000);
  stock(mine, {{STUFF_ORE, 76001}, {STUFF_LABOR, 209}});
  mine.update();
  CHECK(stored[STUFF_ORE] == 76001 && stored[STUFF_LABOR] == 209);
  stock(mine, {{STUFF_ORE, 76000}});
  mine.update();
  CHECK(stored[STUFF_ORE] == 81000 && stored[STUFF_LABOR] == 0);
  CHECK(world.map(site.e())->ore_reserve == 999 && !world.stats.sustainability.mining_flag);

  stock(mine, {{STUFF_ORE, 85000}, {STUFF_LABOR, 209}});
  mine.update();
  CHECK(stored[STUFF_ORE] == 85000 && stored[STUFF_LABOR] == 209);
  stock(mine, {{STUFF_ORE, 85001}, {STUFF_LABOR, 208}});
  mine.update();
  CHECK(stored[STUFF_ORE] == 85001 && stored[STUFF_LABOR] == 208);
  CHECK(world.map(site)->ore_reserve == 0);
  stock(mine, {{STUFF_LABOR, 209}});
  mine.update();
  CHECK(stored[STUFF_ORE] == 80001 && stored[STUFF_LABOR] == 0);
  CHECK(world.map(site)->ore_reserve == 1 && world.map(site.e())->ore_reserve == 999);
}

void oreMineExhaustion() {
  World world(32);
  world.map(site.e())->ore_reserve = 1;
  auto& mine = place(world, oremineConstructionGroup);
  stock(mine, {{STUFF_LABOR, 209}});
  mine.update();
  CHECK(mine.flags & FLAG_EVACUATE);
  CHECK(mine.commodityCount[STUFF_ORE] == 5000 && world.map(site.e())->ore_reserve == 0);
  CHECK(world.map(site)->getConstruction() == &mine);

  stock(mine, {{STUFF_ORE, 0}});
  mine.update();
  CHECK(!world.map(site)->getConstruction());
  for(int y = 0; y < GROUP_OREMINE_SIZE; ++y) {
    for(int x = 0; x < GROUP_OREMINE_SIZE; ++x) {
      const auto* tile = world.map(site.e(x).s(y));
      CHECK(!tile->getReportingConstruction() && tile->is_water());
      CHECK(tile->flags & FLAG_HAS_UNDERGROUND_WATER);
    }
  }
  world.do_time_step();
}

void communeProduction() {
  World world(32);
  auto& commune = place(world, communeConstructionGroup);
  auto& stored = commune.commodityCount;
  world.total_time = 1;
  stock(commune, {{STUFF_WATER, 79}, {STUFF_WASTE, 59}});
  commune.update();
  CHECK(stored[STUFF_COAL] == 1 && stored[STUFF_ORE] == 6);
  CHECK(stored[STUFF_WATER] == 79 && stored[STUFF_WASTE] == 59);

  stock(commune, {{STUFF_WATER, 80}, {STUFF_WASTE, 60}, {STUFF_ORE, 134}});
  world.total_time = 10;
  world.map(site)->pollution = 2;
  world.map(site.e())->pollution = 2;
  commune.update();
  CHECK(stored[STUFF_COAL] == 4 && stored[STUFF_ORE] == 140);
  CHECK(stored[STUFF_WATER] == 0 && stored[STUFF_WASTE] == 40 && stored[STUFF_STEEL] == 2);
  CHECK(world.map(site)->pollution == 2 && world.map(site.e())->pollution == 1);

  stock(commune, {{STUFF_COAL, 60}, {STUFF_ORE, 139}, {STUFF_WASTE, 60}});
  world.total_time = 20;
  commune.update();
  CHECK(stored[STUFF_COAL] == 60 && stored[STUFF_ORE] == 140 && stored[STUFF_STEEL] == 2);
  CHECK(world.map(site)->pollution == 1 && world.map(site.e())->pollution == 0);
}

void idleCommuneBecomesParks() {
  World world(32);
  world.map(site.e())->flags |= FLAG_HAS_UNDERGROUND_WATER;
  auto& commune = place(world, communeConstructionGroup);
  stock(commune, {{STUFF_COAL, 60}, {STUFF_ORE, 140}, {STUFF_STEEL, 40}});
  for(int month = 0; month < 120; ++month) {
    world.total_time = 100 * month + 99;
    commune.update();
  }
  CHECK(world.map(site)->getConstruction() == &commune);
  stock(commune, {{STUFF_ORE, 134}});
  world.total_time = 12099;
  commune.update();
  CHECK(commune.commodityCount[STUFF_ORE] == 140);
  world.total_time = 12199;
  commune.update();
  CHECK(world.map(site)->getConstruction() == &commune);
  world.total_time = 12299;
  commune.update();
  CHECK(!world.map(site)->getConstruction());
  CHECK(world.map(site.e())->getGroup() == GROUP_PARKLAND);
  CHECK(!world.map(site.s())->getConstruction());
  world.do_time_step();
}

void farmWaterAndCapacity() {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  for(int x = 0; x < 4; ++x) {
    world.map(site.e(x))->flags |= FLAG_HAS_UNDERGROUND_WATER;
    world.map(site.e(x).s())->flags |= FLAG_HAS_UNDERGROUND_WATER;
  }
  auto& farm = place(world, organic_farmConstructionGroup);
  auto& stored = farm.commodityCount;
  stock(farm, {{STUFF_LABOR, 13}, {STUFF_LOVOLT, 49}, {STUFF_WATER, 500}, {STUFF_WASTE, 17}});
  farm.update();
  CHECK(stored[STUFF_FOOD] == 68 && stored[STUFF_LABOR] == 0);
  CHECK(stored[STUFF_LOVOLT] == 49 && stored[STUFF_WATER] == 500 && stored[STUFF_WASTE] == 17);

  stock(farm, {{STUFF_LABOR, 13}, {STUFF_LOVOLT, 50}, {STUFF_WASTE, 18}});
  farm.update();
  CHECK(stored[STUFF_FOOD] == 1168 && stored[STUFF_LOVOLT] == 0);
  CHECK(stored[STUFF_LABOR] == 0 && stored[STUFF_WATER] == 100 && stored[STUFF_WASTE] == 12);

  stock(farm, {{STUFF_FOOD, 10971}, {STUFF_LABOR, 13}, {STUFF_LOVOLT, 50}});
  farm.update();
  CHECK(stored[STUFF_FOOD] == 10971 && stored[STUFF_LOVOLT] == 50 && stored[STUFF_LABOR] == 13);
  stock(farm, {{STUFF_FOOD, 10970}});
  farm.update();
  CHECK(stored[STUFF_FOOD] == 11000 && stored[STUFF_LOVOLT] == 48 && stored[STUFF_LABOR] == 13);
  CHECK(stored[STUFF_WATER] == 100);
}

void dryFarmMinimumYield() {
  World world(32);
  auto& farm = place(world, organic_farmConstructionGroup);
  world.total_time = 99;
  farm.update();
  CHECK(farm.commodityCount[STUFF_FOOD] == 30);
  CHECK(farm.commodityProdPrev[STUFF_FOOD] == 30);
}

void millFuelAndCapacity() {
  World world(32);
  auto& mill = place(world, millConstructionGroup);
  auto& stored = mill.commodityCount;
  stock(mill, {{STUFF_LABOR, 35}, {STUFF_FOOD, 37}, {STUFF_COAL, 1}, {STUFF_LOVOLT, 60}, {STUFF_GOODS, 1426}});
  mill.update();
  CHECK(stored[STUFF_GOODS] == 1426 && stored[STUFF_LOVOLT] == 60 && stored[STUFF_COAL] == 1);
  stock(mill, {{STUFF_GOODS, 1425}});
  mill.update();
  CHECK(stored[STUFF_GOODS] == 1500 && stored[STUFF_COAL] == 1 && stored[STUFF_LOVOLT] == 0);
  CHECK(stored[STUFF_LABOR] == 0 && stored[STUFF_FOOD] == 0);

  stock(mill, {{STUFF_LABOR, 210}, {STUFF_FOOD, 222}, {STUFF_COAL, 6}, {STUFF_GOODS, 0}});
  for(int day = 0; day < 5; ++day) mill.update();
  CHECK(world.map(site)->pollution == 0 && stored[STUFF_GOODS] == 375);
  mill.update();
  CHECK(world.map(site)->pollution == 1 && stored[STUFF_GOODS] == 450 && stored[STUFF_COAL] == 0);
}

void solarHeadroom() {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  auto& plant = place(world, solarPowerConstructionGroup);
  auto& stored = plant.commodityCount;
  stock(plant, {{STUFF_HIVOLT, 17101}, {STUFF_LABOR, 25}});
  plant.update();
  CHECK(stored[STUFF_HIVOLT] == 17101 && stored[STUFF_LABOR] == 25);
  stock(plant, {{STUFF_HIVOLT, 17100}, {STUFF_LABOR, 24}});
  plant.update();
  CHECK(stored[STUFF_HIVOLT] == 17100 && stored[STUFF_LABOR] == 24);
  stock(plant, {{STUFF_LABOR, 25}});
  plant.update();
  CHECK(stored[STUFF_HIVOLT] == 18000 && stored[STUFF_LABOR] == 0);
}

void windHeadroom() {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  world.total_money = 20;
  auto& mill = place(world, windmillConstructionGroup);
  const MapPoint other(20, 20);
  windpowerConstructionGroup.placeItem(world, other);
  auto& plant = *world.map(other)->getConstruction();
  stock(mill, {{STUFF_LOVOLT, 8550}, {STUFF_LABOR, 5}});
  stock(plant, {{STUFF_HIVOLT, 8550}, {STUFF_LABOR, 7}});
  world.total_time = 1;
  mill.update();
  plant.update();
  CHECK(mill.commodityCount[STUFF_LOVOLT] == 9000 && mill.commodityCount[STUFF_LABOR] == 0);
  CHECK(plant.commodityCount[STUFF_HIVOLT] == 8550 && plant.commodityCount[STUFF_LABOR] == 7);
  CHECK(world.stats.expenses.windmill.acc == 2 && world.total_money == 19);

  stock(plant, {{STUFF_HIVOLT, 8549}});
  world.total_time = 2;
  plant.update();
  CHECK(plant.commodityCount[STUFF_HIVOLT] == 9000 && plant.commodityCount[STUFF_LABOR] == 0);
  CHECK(world.stats.expenses.windmill.acc == 2);
}

void powerlineLoss() {
  World world(32);
  auto& line = place(world, powerlineConstructionGroup);
  stock(line, {{STUFF_HIVOLT, 1}});
  line.update();
  world.total_time = 99;
  line.update();
  CHECK(line.commodityCount[STUFF_HIVOLT] == 0 && line.commodityProdPrev[STUFF_HIVOLT] == -1);
}

void heavyIndustryEnergy() {
  struct Case { int hv, lv, coal, steel, hvLeft, lvLeft, coalLeft, laborLeft, pollution; };
  for(const auto& test : {
    Case{3700,  400,  20, 36, 100, 400, 20, 18, 9},
    Case{2000, 3300,  20, 36,   0, 100, 20, 18, 9},
    Case{2000, 1000,  23, 36,   0,   0,  1,  0, 9},
    Case{2000, 1000,  21,  7, 2000, 1000, 21, 18, 1},
  }) {
    World world(32);
    auto& industry = place(world, industryHeavyConstructionGroup);
    auto& stored = industry.commodityCount;
    stock(industry, {{STUFF_ORE, 612}, {STUFF_LABOR, 46}, {STUFF_HIVOLT, test.hv},
      {STUFF_LOVOLT, test.lv}, {STUFF_COAL, test.coal}, {STUFF_STEEL, 685}});
    industry.update();
    CHECK(stored[STUFF_STEEL] == 685 && stored[STUFF_ORE] == 612 && stored[STUFF_LABOR] == 46);
    stock(industry, {{STUFF_STEEL, 684}});
    industry.update();
    CHECK(stored[STUFF_STEEL] == 684 + test.steel && stored[STUFF_ORE] == 0);
    CHECK(stored[STUFF_HIVOLT] == test.hvLeft && stored[STUFF_LOVOLT] == test.lvLeft);
    CHECK(stored[STUFF_COAL] == test.coalLeft && stored[STUFF_LABOR] == test.laborLeft);
    CHECK(world.map(site)->pollution == test.pollution);
  }
}

void lightIndustryEnergy() {
  struct Case { int hv, lv, goods, hvLeft, lvLeft, oreLeft; };
  for(const auto& test : {
    Case{0, 0, 450, 0, 0, 125},
    Case{5000, 8001, 1800, 0, 1, 0},
    Case{3500, 11001, 1800, 1, 0, 0},
  }) {
    World world(32);
    auto& industry = place(world, industryLightConstructionGroup);
    auto& stored = industry.commodityCount;
    stock(industry, {{STUFF_LABOR, 59}, {STUFF_ORE, 250}, {STUFF_STEEL, 12},
      {STUFF_HIVOLT, test.hv}, {STUFF_LOVOLT, test.lv}});
    industry.update();
    CHECK(stored[STUFF_GOODS] == test.goods && stored[STUFF_ORE] == test.oreLeft);
    CHECK(stored[STUFF_LABOR] == 0 && stored[STUFF_STEEL] == 0);
    CHECK(stored[STUFF_HIVOLT] == test.hvLeft && stored[STUFF_LOVOLT] == test.lvLeft);
    CHECK(world.map(site)->pollution == 11);
  }
}

void lightIndustryCapacity() {
  World world(32);
  auto& industry = place(world, industryLightConstructionGroup);
  auto& stored = industry.commodityCount;
  world.total_time = 99;
  stock(industry, {{STUFF_LABOR, 59}, {STUFF_ORE, 250}, {STUFF_STEEL, 12},
    {STUFF_LOVOLT, 18000}, {STUFF_GOODS, 35776}});
  industry.update();
  CHECK(stored[STUFF_GOODS] == 35776 && stored[STUFF_LABOR] == 59);

  stock(industry, {{STUFF_GOODS, 35775}});
  industry.update();
  CHECK(stored[STUFF_GOODS] == 36000 && stored[STUFF_STEEL] == 12 && stored[STUFF_LOVOLT] == 18000);
  CHECK(stored[STUFF_ORE] == 125 && stored[STUFF_LABOR] == 19);
  CHECK(industry.commodityProdPrev[STUFF_GOODS] == 0);
  world.total_time = 100;
  industry.update();
  CHECK(industry.commodityProdPrev[STUFF_GOODS] == 225);
}

void industryWasteOverflow(ConstructionGroup& group, int labor, int ore, int capacity, int waste, int power) {
  World world(32);
  world.tech_level = 2 * MAX_TECH_LEVEL;
  auto& industry = place(world, group);
  stock(industry, {{STUFF_LABOR, labor}, {STUFF_ORE, ore}, {STUFF_HIVOLT, power}, {STUFF_WASTE, capacity - 1}});
  industry.update();
  CHECK(industry.commodityCount[STUFF_WASTE] == capacity);
  CHECK(world.map(site)->pollution == waste - 1);
}

void potteryCooldown() {
  World world(32);
  auto& pottery = place(world, potteryConstructionGroup);
  auto& stored = pottery.commodityCount;
  stock(pottery, {{STUFF_GOODS, 666}, {STUFF_LABOR, 35}, {STUFF_COAL, 2}, {STUFF_ORE, 11}});
  pottery.update();
  CHECK(stored[STUFF_GOODS] == 666 && stored[STUFF_LABOR] == 35);
  stock(pottery, {{STUFF_GOODS, 665}});
  for(int day = 1; day <= 25; ++day) {
    world.total_time = day;
    pottery.update();
    CHECK(stored[STUFF_GOODS] == 665 && stored[STUFF_COAL] == 2 && stored[STUFF_ORE] == 11);
  }
  world.total_time = 26;
  pottery.update();
  CHECK(stored[STUFF_GOODS] == 700 && stored[STUFF_LABOR] == 0);
  CHECK(stored[STUFF_COAL] == 0 && stored[STUFF_ORE] == 0 && world.map(site)->pollution == 1);
}

void recycleCapacity() {
  World world(32);
  world.tech_level = 2 * MAX_TECH_LEVEL;
  world.total_money = -2000000000;
  auto& recycle = place(world, recycleConstructionGroup);
  auto& stored = recycle.commodityCount;
  stock(recycle, {{STUFF_WASTE, 9001}, {STUFF_LABOR, 34}, {STUFF_LOVOLT, 250},
    {STUFF_STEEL, 159}, {STUFF_ORE, 7999}});
  recycle.update();
  CHECK(world.total_money == -2000000000 && world.stats.expenses.recycle.acc == 0);
  CHECK(stored[STUFF_WASTE] == 8951 && stored[STUFF_LABOR] == 34 && stored[STUFF_LOVOLT] == 250);
  CHECK(stored[STUFF_ORE] == 7999 && stored[STUFF_STEEL] == 159);

  world.total_money = 9;
  stock(recycle, {{STUFF_WASTE, 9500}, {STUFF_LOVOLT, 249}});
  recycle.update();
  CHECK(world.total_money == 6 && stored[STUFF_WASTE] == 9450 && stored[STUFF_ORE] == 7999);
  stock(recycle, {{STUFF_WASTE, 9500}, {STUFF_LOVOLT, 250}});
  recycle.update();
  CHECK(world.total_money == 3 && world.stats.expenses.recycle.acc == 6);
  CHECK(stored[STUFF_WASTE] == 9000 && stored[STUFF_LABOR] == 0 && stored[STUFF_LOVOLT] == 0);
  CHECK(stored[STUFF_ORE] == 8000 && stored[STUFF_STEEL] == 160);

  stock(recycle, {{STUFF_WASTE, 500}, {STUFF_LABOR, 34}, {STUFF_LOVOLT, 250},
    {STUFF_STEEL, 0}, {STUFF_ORE, 0}});
  recycle.update();
  CHECK(stored[STUFF_WASTE] == 0 && stored[STUFF_ORE] == 400 && stored[STUFF_STEEL] == 8);
  CHECK(world.total_money == 0 && world.map(site)->pollution == 0);
}
}

int main() {
  try {
    coalPower();
    coalMineThresholds();
    coalMineFullDeposit();
    coalMineExhaustion();
    oreMineConstruction();
    oreMineThresholds();
    oreMineExhaustion();
    communeProduction();
    idleCommuneBecomesParks();
    farmWaterAndCapacity();
    dryFarmMinimumYield();
    millFuelAndCapacity();
    solarHeadroom();
    windHeadroom();
    powerlineLoss();
    heavyIndustryEnergy();
    lightIndustryEnergy();
    lightIndustryCapacity();
    industryWasteOverflow(industryHeavyConstructionGroup, 46, 612, 180, 9, 3600);
    industryWasteOverflow(industryLightConstructionGroup, 40, 125, 1800, 11, 0);
    potteryCooldown();
    recycleCapacity();
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
