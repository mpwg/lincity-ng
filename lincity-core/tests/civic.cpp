#include "lincity/messages.hpp"
#include "lincity/modules/fire.hpp"
#include "lincity/modules/market.hpp"
#include "lincity/modules/modules_interfaces.hpp"
#include "lincity/modules/monument.hpp"
#include "lincity/modules/parkland.hpp"
#include "lincity/modules/residence.hpp"
#include "lincity/modules/shanty.hpp"
#include "lincity/modules/tip.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"
#include "util/simulation_trace.hpp"

#include <cstdlib>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <utility>

namespace {
void require(bool condition, const char* description) {
  if(!condition) throw std::runtime_error(description);
}

template<typename Building>
Building& place(World& world, ConstructionGroup& group, MapPoint site) {
  group.placeItem(world, site);
  auto* building = dynamic_cast<Building*>(world.map(site)->getConstruction());
  require(building, "placed construction has the wrong type");
  return *building;
}

template<typename Operation>
void replayChoices(const char* events, Operation operation) {
  std::istringstream input(std::string("LINCITY-SIMULATION-TRACE 1\n") + events + "Z\n");
  SimulationTrace trace(input);
  operation();
  trace.finish();
}

void verifyPark(ConstructionGroup& group) {
  World world(32);
  const MapPoint site(12, 12);
  world.tech_level = MAX_TECH_LEVEL;
  world.total_money = 1000000;
  Message::ptr message;
  require(!group.can_build_here(world, site, message) &&
          dynamic_cast<const DesertHereMessage*>(message.get()),
          "park could be built without groundwater");
  world.map(site)->flags |= FLAG_HAS_UNDERGROUND_WATER;
  world.buildConstruction(group, site);
  auto& park = *world.map(site)->getConstruction();
  world.map(site)->pollution = 11;
  world.map(site.e())->pollution = 11;

  world.total_time = 1;
  park.update();
  require(world.map(site)->pollution == 11, "park removed pollution on an odd day");
  world.total_time = 2;
  park.update();
  require(world.map(site)->pollution == 10, "park did not remove one unit on an even day");
  require(world.map(site.e())->pollution == 11, "park altered pollution on an adjacent tile");
  world.total_time = 4;
  park.update();
  require(world.map(site)->pollution == 10, "park reduced pollution below its floor");
}

void verifyMonument() {
  World world(32);
  auto& monument = place<Monument>(world, monumentConstructionGroup, MapPoint(12, 12));
  Message::ptr message;
  world.tech_level = 40000;
  world.total_time = 1;
  monument.levelStuff(STUFF_LABOR, MONUMENT_GET_LABOR);
  monument.update();
  require(monument.labor_consumed == 0 &&
          monument.commodityCount[STUFF_LABOR] == MONUMENT_GET_LABOR &&
          world.tech_level == 40000, "unfinished monument used its minimum inventory or made tech");
  require(!monument.can_bulldoze(message) &&
          dynamic_cast<const CannotBulldozeIncompleteMonumentMessage*>(message.get()),
          "unfinished monument could be bulldozed");

  world.total_time = 2;
  for(int labor = 0; labor < BUILD_MONUMENT_LABOR; labor += MONUMENT_GET_LABOR) {
    monument.levelStuff(STUFF_LABOR, MONUMENT_GET_LABOR + 1);
    monument.update();
  }
  require(monument.completion == 100 && monument.labor_consumed == BUILD_MONUMENT_LABOR &&
          monument.commodityCount[STUFF_LABOR] == 1 && !monument.completed,
          "the final labor batch should reach 100 percent before completion is applied");
  monument.update();
  require(monument.completed && monument.can_bulldoze(message), "finished monument was not usable");
  constexpr unsigned int evacuation = FLAG_EVACUATE | FLAG_NEVER_EVACUATE;
  require((monument.flags & evacuation) == evacuation && monument.commodityCount[STUFF_LABOR] == 1,
          "finished monument lost its surplus labor or evacuation flags");
  require(world.tech_level == 40000, "monument awarded tech outside its scheduled days");

  for(int interval = 1; interval <= 3; ++interval) {
    world.total_time = 1 + interval * MONUMENT_DAYS_PER_TECH;
    monument.update();
    require(world.tech_level == 40000 + (interval == 3),
            "monument research ignored its technology-dependent interval");
  }
  require(monument.tech_made == 1, "monument did not track its technology contribution");
  world.tech_level = MONUMENT_TECH_EXPIRE * 1000;
  world.total_time += MONUMENT_DAYS_PER_TECH;
  monument.update();
  require(world.tech_level == MONUMENT_TECH_EXPIRE * 1000 && monument.tail_off == 0,
          "monument kept researching at its expiration level");
}

void verifyTip() {
  World world(32);
  auto& tip = place<Tip>(world, tipConstructionGroup, MapPoint(12, 12));
  Message::ptr message;
  require(tip.can_bulldoze(message), "empty tip refused demolition");

  const int critical = TIP_TAKES_WASTE * CRITICAL_WASTE_LEVEL / 100;
  const int buryThreshold = TIP_TAKES_WASTE * (CRITICAL_WASTE_LEVEL + 1) / 100;
  tip.levelStuff(STUFF_WASTE, buryThreshold - 1);
  tip.update();
  require(tip.total_waste == 0 && world.stats.sustainability.mining_flag,
          "tip buried waste at the rounded critical percentage");

  tip.levelStuff(STUFF_WASTE, buryThreshold);
  tip.update();
  require(tip.total_waste == WASTE_BURRIED &&
          tip.commodityCount[STUFF_WASTE] == buryThreshold - WASTE_BURRIED,
          "tip did not bury one batch above the critical percentage");
  require(!world.stats.sustainability.mining_flag, "burial did not reset mining sustainability");
  require(!tip.can_bulldoze(message) &&
          dynamic_cast<const CannotBulldozeNonemptyTipMessage*>(message.get()),
          "filled tip was bulldozable");

  tip.degration_days = TIP_DEGRADE_TIME - WASTE_BURRIED;
  tip.levelStuff(STUFF_WASTE, critical);
  tip.update();
  require(tip.total_waste == WASTE_BURRIED - 1 && tip.degration_days == 0 &&
          tip.commodityCount[STUFF_WASTE] == critical,
          "tip should degrade one unit while leaving the inbox unchanged at the critical percentage");

  tip.levelStuff(STUFF_WASTE, critical - 1);
  world.total_time = 99;
  tip.update();
  require(tip.total_waste == 0 &&
          tip.commodityCount[STUFF_WASTE] == critical - 1 + WASTE_BURRIED - 1,
          "tip did not excavate its remaining waste below the critical percentage");
  require(tip.can_bulldoze(message) && tip.busy == 2,
          "emptied tip should be bulldozable and report only burial and excavation as operating days");

  tip.total_waste = MAX_WASTE_AT_TIP;
  tip.degration_days = 0;
  tip.levelStuff(STUFF_WASTE, buryThreshold);
  tip.update();
  require(tip.total_waste == MAX_WASTE_AT_TIP - MAX_WASTE_AT_TIP / TIP_DEGRADE_TIME &&
          tip.commodityCount[STUFF_WASTE] == buryThreshold,
          "near-full tip buried a batch that did not fit after degradation");
}

void verifyResidence() {
  World world(32);
  world.people_pool = 0;
  const MapPoint site(12, 12);
  auto& home = place<Residence>(world, residenceMLConstructionGroup, site);
  constexpr int tenants = 24;
  constexpr int householdPower = POWER_RES_OVERHEAD + POWER_USE_PER_PERSON * tenants;
  home.local_population = tenants;
  world.map(site)->flags |= FLAG_HEALTH_COVER | FLAG_FIRE_COVER | FLAG_CRICKET_COVER;
  home.levelStuff(STUFF_FOOD, tenants);
  home.levelStuff(STUFF_WATER, tenants);
  home.levelStuff(STUFF_LOVOLT, householdPower + tenants / 2);
  home.levelStuff(STUFF_GOODS, 2 * (tenants / 4));

  replayChoices("C 1\nC 1\nC 500\n", [&] { home.update(); });
  require((home.flags & (FLAG_FED | FLAG_HAD_POWER | FLAG_EMPLOYED)) ==
          (FLAG_FED | FLAG_HAD_POWER | FLAG_EMPLOYED), "supplied residence missed a service flag");
  for(Commodity input : {STUFF_FOOD, STUFF_WATER, STUFF_LOVOLT, STUFF_GOODS})
    require(home.commodityCount[input] == 0, "supplied residence kept some of its daily inputs");
  require(home.commodityCount[STUFF_LABOR] == 17 && home.commodityCount[STUFF_WASTE] == 4,
          "supplied residence produced the wrong labor or household waste");
  require(home.desireability == 105 && home.local_population == tenants,
          "supplied residence has incorrect desirability or population");
  auto& population = world.stats.population;
  require(population.housed_m.acc == tenants && population.population_m.acc == tenants &&
          population.housing_m.acc == GROUP_RESIDENCE_ML_MAX_POP,
          "housing statistics did not record the daily occupancy and capacity");

  const int laborCapacity = residenceMLConstructionGroup.commodityRuleCount[STUFF_LABOR].maxload;
  home.levelStuff(STUFF_LABOR, laborCapacity);
  home.levelStuff(STUFF_FOOD, tenants);
  home.levelStuff(STUFF_WATER, tenants - 1);
  replayChoices("C 1\nC 1\nC 500\n", [&] { home.update(); });
  require(!(home.flags & (FLAG_FED | FLAG_EMPLOYED)) && (home.flags & FLAG_HAD_POWER),
          "water shortage or full labor inventory did not clear the daily flags");
  require(home.commodityCount[STUFF_LABOR] == laborCapacity &&
          home.commodityCount[STUFF_FOOD] == tenants &&
          home.commodityCount[STUFF_WATER] == tenants - 1,
          "hungry or unemployed residence consumed part of the ration or overproduced labor");
  require(home.local_population == tenants - 1 && population.starving_m.acc == tenants - 1 &&
          population.unemployed_m.acc == tenants - 1 &&
          population.unemployed_days_t == tenants - 1 && population.starve_deaths_t == 1 &&
          population.deaths_m.acc == 1 && population.unnat_deaths_m.acc == 1,
          "starvation and unemployment statistics did not account for surviving tenants");

  home.bulldoze();
  world.do_time_step();
  require(world.people_pool == tenants - 1, "demolition did not return every tenant to the pool");
}

void verifyResidenceFire() {
  World world(32);
  world.people_pool = 0;
  const MapPoint site(12, 12);
  auto& home = place<Residence>(world, residenceLLConstructionGroup, site);
  home.local_population = 5;
  home.torch();
  require(world.stats.population.deaths_m.acc == 2 && world.stats.population.unnat_deaths_m.acc == 2,
          "residential fire counted the wrong number of casualties");
  require(world.people_pool == 3 && world.map(site)->getGroup() == GROUP_FIRE,
          "residential fire did not return survivors to the population pool");
  world.do_time_step();
  require(world.people_pool == 3, "fire survivors were counted again when the residence was destroyed");
}

void verifyShantyWaste() {
  World world(32);
  const MapPoint site(12, 12);
  auto& shanty = place<Shanty>(world, shantyConstructionGroup, site);
  constexpr std::pair<Commodity, int> stocks[] = {
    {STUFF_FOOD, SHANTY_GET_FOOD}, {STUFF_LABOR, SHANTY_GET_LABOR},
    {STUFF_GOODS, SHANTY_GET_GOODS}, {STUFF_COAL, SHANTY_GET_COAL},
    {STUFF_ORE, SHANTY_GET_ORE}, {STUFF_STEEL, SHANTY_GET_STEEL}
  };
  for(const auto& [commodity, amount] : stocks)
    shanty.levelStuff(commodity, amount - 1);
  shanty.update();
  for(const auto& [commodity, amount] : stocks)
    require(shanty.commodityCount[commodity] == amount - 1, "shanty consumed an incomplete batch");
  require(shanty.commodityCount[STUFF_WASTE] == 2 * SHANTY_PUT_WASTE,
          "shanty omitted one of its two daily waste deposits");

  for(const auto& [commodity, amount] : stocks)
    shanty.levelStuff(commodity, amount);
  world.taxable.labor = 2 * SHANTY_GET_LABOR + 1;
  world.taxable.goods = 2 * SHANTY_GET_GOODS - 1;
  world.taxable.coal = 0;
  shanty.update();
  for(const auto& [commodity, amount] : stocks)
    require(shanty.commodityCount[commodity] == 0, "shanty failed to consume one batch");
  require(world.taxable.labor == 1 && world.taxable.goods == 0 && world.taxable.coal == 0,
          "shanty applied the wrong reductions or produced negative taxable income");
  require(shanty.commodityCount[STUFF_WASTE] == 4 * SHANTY_PUT_WASTE + SHANTY_GET_GOODS / 3,
          "shanty produced the wrong waste from goods");

  shanty.levelStuff(STUFF_WASTE, MAX_WASTE_AT_SHANTY - 2 * SHANTY_PUT_WASTE);
  world.map(site)->flags |= FLAG_FIRE_COVER;
  replayChoices("S shanty.waste-fire 1\n", [&] { shanty.update(); });
  require(shanty.commodityCount[STUFF_WASTE] == 0 && shanty.start_burning_waste &&
          world.map(site.s().e())->pollution == MAX_WASTE_AT_SHANTY &&
          world.map(site)->pollution == 0,
          "shanty did not burn its full waste inventory on the southeast tile");
  require(world.map(site)->getConstruction() == &shanty,
          "shanty ignited during waste burning despite fire coverage");
}

void verifyShantyPopulation() {
  World world(32);
  world.people_pool = SHANTY_MIN_PP + SHANTY_POP;
  update_shanty(world);
  require(world.stats.groupCount[GROUP_SHANTY] == 0, "a shanty appeared without a market");

  marketConstructionGroup.placeItem(world, MapPoint(16, 16));
  --world.people_pool;
  update_shanty(world);
  require(world.stats.groupCount[GROUP_SHANTY] == 0, "a shanty appeared below the population threshold");
  ++world.people_pool;
  update_shanty(world);
  require(world.stats.groupCount[GROUP_SHANTY] == 1, "a market did not attract a shanty at the threshold");

  shantyConstructionGroup.placeItem(world, MapPoint(5, 5));
  world.people_pool = 7;
  update_shanty(world);
  require(world.people_pool == 0 && world.stats.population.deaths_m.acc == 7,
          "shanty population loss was not bounded by the available population");
  require(world.stats.groupCount[GROUP_SHANTY] == 1 &&
          world.stats.groupCount[GROUP_FIRE] == GROUP_SHANTY_SIZE * GROUP_SHANTY_SIZE,
          "excess shanty was not replaced with a fire on each footprint tile");
  update_shanty(world);
  require(world.stats.groupCount[GROUP_SHANTY] == 1,
          "the last shanty should survive a low population pool");
}
}

int main() {
  std::srand(724);
  BasicUrbg::get().seed(724);
  try {
    verifyPark(parklandConstructionGroup);
    verifyPark(parkpondConstructionGroup);
    verifyMonument();
    verifyTip();
    verifyResidence();
    verifyResidenceFire();
    verifyShantyWaste();
    verifyShantyPopulation();
  } catch(const Message::Exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
  return 0;
}
