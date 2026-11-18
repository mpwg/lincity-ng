#include "lincity/ConstructionReport.hpp"
#include "lincity/UserOperation.hpp"
#include "lincity/messages.hpp"
#include "lincity/modules/blacksmith.hpp"
#include "lincity/modules/cricket.hpp"
#include "lincity/modules/fire.hpp"
#include "lincity/modules/firestation.hpp"
#include "lincity/modules/health_centre.hpp"
#include "lincity/modules/market.hpp"
#include "lincity/modules/modules_interfaces.hpp"
#include "lincity/modules/monument.hpp"
#include "lincity/modules/parkland.hpp"
#include "lincity/modules/residence.hpp"
#include "lincity/modules/shanty.hpp"
#include "lincity/modules/tip.hpp"
#include "lincity/modules/track_road_rail.hpp"
#include "lincity/modules/university.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"
#include "util/simulation_trace.hpp"

#include <cmath>
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
void replayChoices(const std::string& events, Operation operation) {
  std::istringstream input(std::string("LINCITY-SIMULATION-TRACE 1\n") + events + "Z\n");
  SimulationTrace trace(input);
  operation();
  trace.finish();
}

class Inspection : public ConstructionReport {
public:
  std::pair<int, int> progress;
  double percentage = 0.0;
  std::string foodDirection;
  std::pair<int, int> food;

  void addBlank() override {}
  void addLabel(const std::string&) override {}
  void addLabeledText(const std::string&, const std::string&) override {}
  void addInteger(const std::string&, int) override {}
  void addDecimal(const std::string&, double) override {}
  void addPercentage(const std::string&, double value) override { percentage = value; }
  void addQuantity(const std::string&, int value, int capacity) override {
    progress = {value, capacity};
  }
  void addPrefixedQuantity(const std::string& prefix, const std::string& label,
                          int value, int capacity) override {
    if(label == commodityName(STUFF_FOOD)) {
      foodDirection = prefix;
      food = {value, capacity};
    }
  }
};

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
  monument.animate(0);
  require(monument.getFrame().resourceGroup == ResourceGroup::resMap.at("Monument") &&
          monument.getFrame().frame == 0, "completed monument did not switch to its finished appearance");
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
  Inspection report;
  monument.report(report, false);
  require(std::abs(report.percentage - 100.0 / MAX_TECH_LEVEL) < 1e-12,
          "completed monument report did not show the technology it created");
  world.tech_level = MONUMENT_TECH_EXPIRE * 1000;
  world.total_time += MONUMENT_DAYS_PER_TECH;
  monument.update();
  require(world.tech_level == MONUMENT_TECH_EXPIRE * 1000 && monument.tail_off == 0,
          "monument kept researching at its expiration level");

  monument.levelStuff(STUFF_LABOR, 0);
  world.total_time = 99;
  monument.update();
  require(monument.busy == BUILD_MONUMENT_LABOR / MONUMENT_GET_LABOR &&
          monument.working_days == 0, "monument lost its count of construction days");
}

void verifyTip() {
  World world(32);
  const MapPoint site(12, 12);
  auto& tip = place<Tip>(world, tipConstructionGroup, site);
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
  try {
    world.bulldozeArea(site.se());
    throw std::runtime_error("bulldozing a filled waste tip unexpectedly succeeded");
  } catch(const CannotBulldozeNonemptyTipMessage::Exception& error) {
    const auto refusal = error.getMessage();
    require(refusal->getPoint() == site && &refusal->getGroup() == &tipConstructionGroup &&
            refusal->str() == "cannot bulldoze a tip that is full of waste: (12,12)" &&
            world.total_money == 0 && world.map(site)->getConstruction() == &tip,
            "blocked demolition did not identify the tip or altered the city");
  }

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

void verifyFireLifetime(bool covered) {
  World world(32);
  const MapPoint site(12, 12);
  auto& fire = place<Fire>(world, fireConstructionGroup, site);
  if(covered)
    world.map(site)->flags |= FLAG_FIRE_COVER;
  fire.burning_days = FIRE_LENGTH - 1;
  fire.days_before_spread = 2;
  const int dailyProgress = covered ? 5 : 1;
  const int initialMoney = world.total_money;

  Inspection report;
  fire.report(report, false);
  require(report.progress == std::pair{FIRE_LENGTH - 1, FIRE_LENGTH},
          "active fire report did not show progress through its burning phase");
  try {
    world.bulldozeArea(site);
    throw std::runtime_error("demolition removed an active fire");
  } catch(const CannotBulldozeThisEverMessage::Exception&) {
    require(world.map(site)->getConstruction() == &fire && world.total_money == initialMoney,
            "rejected fire demolition altered the site or charged the city");
  }

  fire.burning_days = FIRE_LENGTH;
  fire.torch();
  fire.update();
  require(fire.burning_days == FIRE_LENGTH + dailyProgress &&
          fire.smoking_days == 0 && world.map(site)->pollution == 1,
          "fire did not finish its last burning day at the expected rate");

  fire.getFrame().frame = fire.getFrame().resourceGroup->frameCount - 1;
  replayChoices("C 175\n", [&] { fire.animate(1000); });
  require(fire.getFrame().frame == 0, "burning fire did not loop its animation");
  replayChoices("C 7\n", [&] { fire.update(); });
  require(fire.smoking_days == 7 + dailyProgress && world.map(site)->pollution == 1,
          "smoldering fire did not advance without producing new pollution");
  fire.report(report, false);
  require(report.progress == std::pair{7 + dailyProgress, AFTER_FIRE_LENGTH},
          "smoldering report did not show progress through its smoke phase");
  fire.animate(1000);
  require(fire.getFrame().resourceGroup == ResourceGroup::resMap.at("FireWasteLand") &&
          fire.getFrame().frame == 0, "smoldering fire did not start its wasteland animation");

  fire.smoking_days = AFTER_FIRE_LENGTH - dailyProgress;
  fire.update();
  fire.animate(1000);
  require(world.map(site)->getConstruction() == &fire &&
          fire.smoking_days == AFTER_FIRE_LENGTH && fire.getFrame().frame == 4,
          "burnt site disappeared before reaching its final smoke stage");
  fire.update();
  require(world.map(site)->is_bare() && world.stats.groupCount[GROUP_FIRE] == 0,
          "finished smoke did not clear the building site");

  world.buildConstruction(residenceLLConstructionGroup, site);
  world.do_time_step();
  require(world.map(site)->getGroup() == GROUP_RESIDENCE_LL && world.map.constructionCount() == 1,
          "removing an old fire disturbed the building that replaced it");
}

void verifyFireSpreadBarriers() {
  World world(32);
  const MapPoint site(1, 12);
  auto& fire = place<Fire>(world, fireConstructionGroup, site);
  parklandConstructionGroup.placeItem(world, site.n());
  parklandConstructionGroup.placeItem(world, site.s());
  world.map(site.n())->flags |= FLAG_FIRE_COVER;
  fire.burning_days = 1;

  // West is beyond the city, east is empty, north is protected, and south
  // rolls a value exactly at the target's fire chance.
  const char* draws[] = {"C 3\n", "C 2\n", "C 0\nC 0\n",
                         "C 1\nC 1\n", "C 4\n"};
  for(const char* choices : draws) {
    fire.days_before_spread = 1;
    replayChoices(choices, [&] { fire.update(); });
    require(fire.days_before_spread == FIRE_DAYS_PER_SPREAD &&
            world.stats.groupCount[GROUP_FIRE] == 1 && world.stats.groupCount[GROUP_PARKLAND] == 2,
            "failed fire spread damaged a neighbor or did not reset the spread clock");
  }
}

void verifyResidenceMigration() {
  World world(32);
  world.people_pool = 0;
  auto& home = place<Residence>(world, residenceHHConstructionGroup, MapPoint(12, 12));
  constexpr int tenants = 24;
  home.local_population = tenants;
  home.levelStuff(STUFF_FOOD, tenants);
  home.levelStuff(STUFF_WATER, tenants);

  // Survive, have one birth, and let one resident leave for the population pool.
  replayChoices("C 1\nC 0\nC 0\n", [&] { home.update(); });
  require(home.local_population == tenants && world.people_pool == 1 &&
          world.stats.population.births_t == 1 && world.stats.population.births_m.acc == 1,
          "residence did not account for a birth and an emigrant");

  home.levelStuff(STUFF_FOOD, tenants);
  home.levelStuff(STUFF_WATER, tenants);
  // On the next day the maximum migration draw for this home is 4299.
  replayChoices("C 1\nC 1\nC 4299\n", [&] { home.update(); });
  require(home.local_population == tenants + 1 && world.people_pool == 0 &&
          world.stats.population.births_t == 1, "desirable residence did not attract an immigrant");
}

void verifyResidencePollution() {
  World world(32);
  world.people_pool = 0;
  const MapPoint site(12, 12);
  auto& home = place<Residence>(world, residenceLLConstructionGroup, site);
  constexpr int tenants = 12;
  home.local_population = tenants;
  home.levelStuff(STUFF_FOOD, tenants);
  home.levelStuff(STUFF_WATER, tenants);
  home.levelStuff(STUFF_GOODS, tenants / 4);
  home.levelStuff(STUFF_LOVOLT, POWER_RES_OVERHEAD + POWER_USE_PER_PERSON * tenants);
  world.map(site)->pollution = 1600;

  replayChoices("C 0\nC 94\nC 1\nC 500\n", [&] { home.update(); });
  const auto& population = world.stats.population;
  require(home.local_population == tenants - 1 && population.deaths_m.acc == 1 &&
          population.unnat_deaths_m.acc == 1 && population.pollution_deaths_t == 1 &&
          population.starve_deaths_t == 0, "fatal pollution was not counted separately from starvation");
  require(home.commodityCount[STUFF_LOVOLT] == 0 && home.commodityCount[STUFF_WASTE] == 1,
          "basic power supply should permit only the first round of goods consumption");
}

void verifyResidenceDensity() {
  struct Housing {
    ConstructionGroup& group;
    int capacity;
    int deathInterval;
    int desirability;
  };
  const Housing cases[] = {
    {residenceHLConstructionGroup, 200, 759, 45},
    {residenceLHConstructionGroup, 100, 639, 5},
    {residenceMHConstructionGroup, 200, 791, 5},
  };
  constexpr int tenants = 24;
  for(const auto& housing : cases) {
    World world(32);
    world.people_pool = 0;
    auto& home = place<Residence>(world, housing.group, MapPoint(12, 12));
    home.local_population = tenants;
    home.levelStuff(STUFF_FOOD, tenants);
    home.levelStuff(STUFF_WATER, tenants);
    replayChoices("C 1\nC 1\nC 500\n", [&] { home.update(); });
    require(home.local_population == tenants && home.deaths == housing.deathInterval &&
            home.desireability == housing.desirability &&
            world.stats.population.housing_m.acc == housing.capacity,
            "residence type reported the wrong health, desirability or capacity");
  }
}

void verifyUniversityFunding() {
  World world(32);
  auto& university = place<University>(world, universityConstructionGroup, MapPoint(12, 12));
  constexpr int creditLimit = -2000000000;
  world.total_money = creditLimit + UNIVERSITY_RUNNING_COST - 1;
  university.levelStuff(STUFF_LABOR, UNIVERSITY_LABOR);
  university.levelStuff(STUFF_GOODS, UNIVERSITY_GOODS);
  university.update();
  require(world.total_money == creditLimit + UNIVERSITY_RUNNING_COST - 1 &&
          world.stats.expenses.university.acc == 0 && world.tech_level == 0 &&
          university.commodityCount[STUFF_LABOR] == UNIVERSITY_LABOR &&
          university.commodityCount[STUFF_GOODS] == UNIVERSITY_GOODS &&
          university.commodityCount[STUFF_WASTE] == 0 && !world.popMessage(),
          "university operated when the city could not pay its upkeep");

  ++world.total_money;
  world.total_time = 99;
  university.update();
  require(world.total_money == creditLimit && world.stats.expenses.university.acc == UNIVERSITY_RUNNING_COST &&
          world.tech_level == UNIVERSITY_TECH_MADE && university.busy == 1 &&
          university.commodityCount[STUFF_LABOR] == 0 && university.commodityCount[STUFF_GOODS] == 0 &&
          university.commodityCount[STUFF_WASTE] == UNIVERSITY_GOODS / 3,
          "fully funded university did not resume research");
}

void verifyMonthlyInsolvency(int cash) {
  World world(32);
  constexpr int creditLimit = -2000000000;
  world.total_money = cash;
  world.total_time = NUMOF_DAYS_IN_MONTH - 2;
  world.stats.population.unnat_deaths_m.acc = 1;
  world.stats.population.unemployed_m.acc = 100;
  world.money_rates.dole = 25;
  world.do_time_step();
  require(world.total_time == NUMOF_DAYS_IN_MONTH - 1 && !world.gameEnd,
          "monthly expenses interrupted the city simulation");
  if(cash == creditLimit) {
    require(world.total_money == creditLimit && world.stats.expenses.deaths.acc == 0 &&
            world.stats.expenses.unemployment.acc == 0 && !world.popMessage(),
            "unpayable monthly expenses changed the city balance or reissued a warning");
  } else {
    const auto message = dynamic_message_cast<OutOfMoneyMessage>(world.popMessage());
    require(world.total_money == -25 && world.stats.expenses.deaths.acc == UNNAT_DEATHS_COST &&
            world.stats.expenses.unemployment.acc == 25 && message && !message->isOutOfCredit() &&
            message->str() == "out of money" && !world.popMessage(),
            "city crossing zero did not pay the expenses and report one loss of cash");
  }
}

void verifyAnnualInterest() {
  World world(32);
  const int lastDay = NUMOF_DAYS_IN_YEAR - 1;
  world.total_time = lastDay - 2;
  world.total_money = -100000;
  world.do_time_step();
  require(world.total_money == -100000 && world.stats.expenses.interest.acc == 0,
          "annual interest was charged before the last day of the year");
  world.do_time_step();
  require(world.total_time == lastDay && world.total_money == -101500 &&
          world.stats.expenses.interest.acc == 1500 && !world.popMessage(),
          "year-end settlement charged the wrong interest on city debt");

  World insolvent(32);
  insolvent.total_time = lastDay - 1;
  insolvent.total_money = -2000000000;
  insolvent.do_time_step();
  require(insolvent.total_time == lastDay && insolvent.total_money == -2000000000 &&
          insolvent.stats.expenses.interest.acc == 0 && !insolvent.popMessage(),
          "unpayable interest interrupted the calendar or was booked anyway");
}

void verifyMarketClosedTrade() {
  World world(32);
  const MapPoint site(2, 2);
  auto& market = place<Market>(world, marketConstructionGroup, site);
  market.bootstrap_commodities(70);
  world.total_time = 17;
  market.update();
  require(market.labor == LABOR_MARKET_FULL, "stocked market did not start fully staffed");
  for(Commodity commodity = STUFF_INIT; commodity < STUFF_COUNT; ++commodity) {
    if(market.commodityRuleCount[commodity].maxload)
      require(world.setMarketTrade(site, commodity, false, false), "could not close a commodity trade");
  }

  world.total_time += 25;
  market.update();
  require(market.labor == LABOR_MARKET_EMPTY && market.market_ratio == 0,
          "market with all trades closed did not return to minimal staffing");
  Inspection report;
  market.report(report, false);
  require(report.foodDirection == "---" && report.food == std::pair{70 * MAX_FOOD_IN_MARKET / 100, MAX_FOOD_IN_MARKET},
          "closed market report changed the food amount or showed food moving");
  UserOperation{UserOperation::ACTION_EVACUATE}.execute(world, site.se());
  market.report(report, false);
  require(report.foodDirection == "<< " && report.food == std::pair{70 * MAX_FOOD_IN_MARKET / 100, MAX_FOOD_IN_MARKET},
          "evacuating market report did not show the unchanged food stock as outbound");

  world.total_time = 74;
  world.do_time_step();
  require((world.map(MapPoint(1, 1))->flags & FLAG_MARKET_COVER) &&
          !(world.map(MapPoint(0, 1))->flags & FLAG_MARKET_COVER) &&
          !(world.map(site.e(GROUP_MARKET_RANGE))->flags & FLAG_MARKET_COVER),
          "market coverage did not respect the interior border or distance limit");
}

std::string wasteFireChoices(const char* event, MapPoint site, unsigned int size, int placed = 0) {
  std::ostringstream choices;
  choices << "S " << event << " 1\n";
  for(unsigned int y = 0; y < size; ++y)
    for(unsigned int x = 0; x < size; ++x)
      choices << "P " << ++placed << ' ' << site.x + x << ' ' << site.y + y << ' ' << GROUP_FIRE << '\n';
  return choices.str();
}

void verifyMarketWasteFire(bool covered) {
  World world(32);
  const MapPoint site(12, 12);
  auto& market = place<Market>(world, marketConstructionGroup, site);
  if(covered)
    world.map(site)->flags |= FLAG_FIRE_COVER;
  constexpr int threshold = 85 * MAX_WASTE_IN_MARKET / 100;
  market.levelStuff(STUFF_WASTE, threshold);
  world.total_time = 50;
  market.update();
  require(market.commodityCount[STUFF_WASTE] == threshold && world.map(site.se())->pollution == 0,
          "market burned waste on the excluded fiftieth day");

  world.total_time = 51;
  const auto events = covered ? "S market.waste-fire 1\n" :
    wasteFireChoices("market.waste-fire", site, GROUP_MARKET_SIZE);
  replayChoices(events, [&] { market.update(); });
  require(world.map(site.se())->pollution == MAX_WASTE_IN_MARKET / 20 &&
          market.commodityCount[STUFF_WASTE] == threshold - 7 * MAX_WASTE_IN_MARKET / 10,
          "market burned the wrong amount of waste or polluted the wrong tile");
  if(!covered) {
    const auto message = dynamic_message_cast<FireStartedMessage>(world.popMessage());
    require(message && message->getPoint() == site && &message->getGroup() == &marketConstructionGroup &&
            world.stats.groupCount[GROUP_FIRE] == GROUP_MARKET_SIZE * GROUP_MARKET_SIZE &&
            world.stats.groupCount[GROUP_MARKET] == 0 && !world.popMessage(),
            "unprotected market fire did not replace the full footprint and identify the market");
    return;
  }
  require(world.map(site)->getConstruction() == &market && !world.popMessage(),
          "fire coverage did not prevent a market from igniting");
  const auto& burnFrame = world.map(site)->getFrames()->back();
  const int frameCount = burnFrame.resourceGroup->frameCount;
  for(int step = 0; step <= frameCount; ++step) {
    replayChoices("C 175\n", [&] { market.animate(1000 + step * ANIM_THRESHOLD(200)); });
    require(burnFrame.frame == step % frameCount, "market waste fire did not cycle its animation");
  }
  market.animate(1000 + ANIM_THRESHOLD(6 * WASTE_BURN_TIME));
  require(burnFrame.frame == -1, "market waste fire stayed visible after its animation ended");
}

void verifyOverlappingCoverage() {
  World world(32);
  world.total_money = 1000000;
  const MapPoint stationSite(28, 14);
  auto& station = place<FireStation>(world, fireStationConstructionGroup, stationSite);
  auto& hospital = place<HealthCentre>(world, healthCentreConstructionGroup, MapPoint(24, 22));
  auto& cricket = place<Cricket>(world, cricketConstructionGroup, MapPoint(26, 7));
  station.levelStuff(STUFF_LABOR, FIRESTATION_LABOR);
  station.levelStuff(STUFF_GOODS, FIRESTATION_GOODS);
  auto supplyRemainingServices = [&] {
    hospital.levelStuff(STUFF_LABOR, HEALTH_CENTRE_LABOR);
    hospital.levelStuff(STUFF_GOODS, HEALTH_CENTRE_GOODS);
    cricket.levelStuff(STUFF_LABOR, CRICKET_LABOR);
    cricket.levelStuff(STUFF_GOODS, CRICKET_GOODS);
  };
  supplyRemainingServices();

  const MapPoint lastVisible(30, 15);
  const MapPoint border(31, 15);
  auto& overlapping = world.map(lastVisible)->flags;
  overlapping |= FLAG_HAS_UNDERGROUND_WATER;
  constexpr unsigned int shared = FLAG_FIRE_COVER | FLAG_HEALTH_COVER | FLAG_CRICKET_COVER;
  world.total_time = 74;
  world.do_time_step();
  require((overlapping & shared) == shared && (overlapping & FLAG_HAS_UNDERGROUND_WATER) &&
          !(world.map(border)->flags & shared),
          "overlapping services interfered with each other or reached the outer border");
  require(!(world.map(lastVisible.s())->flags & FLAG_CRICKET_COVER) &&
          !(world.map(MapPoint(30, 7))->flags & FLAG_HEALTH_COVER),
          "municipal services ignored their individual ranges");

  world.bulldozeArea(stationSite);
  supplyRemainingServices();
  world.total_time = DAYS_BETWEEN_COVER + 73;
  world.do_time_step();
  require((overlapping & shared) == shared, "service coverage changed before its renewal day");
  world.do_time_step();
  require((overlapping & shared) == (FLAG_HEALTH_COVER | FLAG_CRICKET_COVER),
          "coverage renewal retained the demolished fire station or cleared a remaining service");
}

void verifyAnnualFireCoverageAfterBuildingLoss() {
  World world(32);
  const MapPoint road(6, 6);
  const MapPoint site(12, 12);
  const int auditDay = 3 * NUMOF_DAYS_IN_YEAR;
  world.total_time = auditDay - 1;
  world.stats.sustainability.fire_years = 3;

  std::ostringstream choices;
  choices << "P 1 " << road.x << ' ' << road.y << ' ' << GROUP_ROAD << '\n'
          << "P 2 " << site.x << ' ' << site.y << ' ' << GROUP_SHANTY << '\n'
          << "D " << auditDay << " 2 1 2\n"
          << wasteFireChoices("shanty.waste-fire", site, GROUP_SHANTY_SIZE, 2);
  replayChoices(choices.str(), [&] {
    roadConstructionGroup.placeItem(world, road);
    auto& shanty = place<Shanty>(world, shantyConstructionGroup, site);
    shanty.levelStuff(STUFF_WASTE, MAX_WASTE_AT_SHANTY - 2 * SHANTY_PUT_WASTE);
    world.do_time_step();
  });
  const auto fire = dynamic_message_cast<FireStartedMessage>(world.popMessage());
  require(fire && fire->getPoint() == site && &fire->getGroup() == &shantyConstructionGroup,
          "fire at the annual audit was not reported");
  require(world.stats.sustainability.fire_years == 6 && world.total_time == auditDay &&
          world.stats.groupCount[GROUP_SHANTY] == 0 &&
          world.stats.groupCount[GROUP_FIRE] == GROUP_SHANTY_SIZE * GROUP_SHANTY_SIZE,
          "fire audit included a lost building, its replacement fires or a road");

  blacksmithConstructionGroup.placeItem(world, MapPoint(20, 20));
  world.total_time = auditDay + NUMOF_DAYS_IN_YEAR - 1;
  world.do_time_step();
  require(world.stats.sustainability.fire_years == 6,
          "ordinary year boundary reset fire protection before the next triennial audit");
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
  shanty.levelStuff(STUFF_LABOR, SHANTY_GET_LABOR);
  shanty.levelStuff(STUFF_COAL, SHANTY_GET_COAL);
  world.taxable.coal = 2 * SHANTY_GET_COAL + 1;
  shanty.update();
  require(world.taxable.labor == 0 && world.taxable.coal == 1,
          "shanty made taxable labor negative or discarded extra taxable coal");

  shanty.levelStuff(STUFF_WASTE, MAX_WASTE_AT_SHANTY - 2 * SHANTY_PUT_WASTE);
  world.map(site)->flags |= FLAG_FIRE_COVER;
  replayChoices("S shanty.waste-fire 1\n", [&] { shanty.update(); });
  require(shanty.commodityCount[STUFF_WASTE] == 0 && shanty.start_burning_waste &&
          world.map(site.s().e())->pollution == MAX_WASTE_AT_SHANTY &&
          world.map(site)->pollution == 0,
          "shanty did not burn its full waste inventory on the southeast tile");
  require(world.map(site)->getConstruction() == &shanty,
          "shanty ignited during waste burning despite fire coverage");
  const auto& burnFrame = world.map(site)->getFrames()->back();
  replayChoices("C 175\n", [&] { shanty.animate(1000); });
  require(burnFrame.frame == 0, "waste burning did not make its fire visible");
  const auto frameCount = burnFrame.resourceGroup->frameCount;
  for(int step = 1; step <= frameCount; ++step) {
    replayChoices("C 175\n", [&] { shanty.animate(1000 + step * ANIM_THRESHOLD(200)); });
    require(burnFrame.frame == step % frameCount, "waste burning animation did not advance in a loop");
  }
  world.do_animate(1000 + ANIM_THRESHOLD(3 * WASTE_BURN_TIME));
  require(burnFrame.frame == -1, "waste fire stayed visible after its animation ended");

  world.map(site)->flags &= ~FLAG_FIRE_COVER;
  shanty.levelStuff(STUFF_WASTE, MAX_WASTE_AT_SHANTY - 2 * SHANTY_PUT_WASTE);
  replayChoices(wasteFireChoices("shanty.waste-fire", site, GROUP_SHANTY_SIZE), [&] { shanty.update(); });
  const auto message = dynamic_message_cast<FireStartedMessage>(world.popMessage());
  require(world.stats.groupCount[GROUP_SHANTY] == 0 &&
          world.stats.groupCount[GROUP_FIRE] == GROUP_SHANTY_SIZE * GROUP_SHANTY_SIZE &&
          message && message->getPoint() == site && &message->getGroup() == &shantyConstructionGroup &&
          !world.popMessage(),
          "uncontrolled waste burning did not replace the shanty with fire and report it");
}

void verifyShantyBulldoze() {
  World world(32);
  const MapPoint site(12, 12);
  world.total_money = 2 * GROUP_SHANTY_BUL_COST;
  place<Shanty>(world, shantyConstructionGroup, site);

  world.bulldozeArea(site.se());
  require(world.stats.groupCount[GROUP_SHANTY] == 0 &&
          world.total_money == GROUP_SHANTY_BUL_COST &&
          world.stats.expenses.construction.acc == GROUP_SHANTY_BUL_COST,
          "shanty demolition did not remove the building and charge exactly once");
  for(const MapPoint tile : {site, site.s(), site.e(), site.se()}) {
    const auto* fire = dynamic_cast<const Fire*>(world.map(tile)->getConstruction());
    require(fire && fire->burning_days == FIRE_LENGTH - 25,
            "bulldozed shanty did not leave a short-lived fire on each tile");
  }
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

void verifyShantyClusterGrowth() {
  World world(32);
  const MapPoint existing(12, 12);
  place<Shanty>(world, shantyConstructionGroup, existing);
  world.people_pool = SHANTY_MIN_PP + 2 * SHANTY_POP + 5;

  const MapPoint vacant(10, 13);
  std::ostringstream choices;
  choices << "C 12\nC 12\nC 1\nP 1 " << vacant.x << ' ' << vacant.y << ' ' << GROUP_SHANTY << '\n';
  replayChoices(choices.str(), [&] { update_shanty(world); });
  require(world.stats.groupCount[GROUP_SHANTY] == 2 && world.map(existing)->getGroup() == GROUP_SHANTY &&
          world.map(vacant.se())->getGroup() == GROUP_SHANTY &&
          world.people_pool == SHANTY_MIN_PP + 2 * SHANTY_POP,
          "a growing shanty cluster did not occupy the nearby vacant plot");
}

void verifyShantyOnFullMap(bool hasShanty) {
  World world(8);
  const MapPoint market(2, 2);
  const MapPoint shanty(4, 4);
  marketConstructionGroup.placeItem(world, market);
  if(hasShanty)
    shantyConstructionGroup.placeItem(world, shanty);
  for(int y = 1; y < world.map.len() - 1; ++y) {
    for(int x = 1; x < world.map.len() - 1; ++x) {
      const MapPoint site(x, y);
      if(world.map(site)->is_bare())
        parklandConstructionGroup.placeItem(world, site);
    }
  }

  world.tech_level = GROUP_HEALTH_TECH + 1;
  world.people_pool = SHANTY_MIN_PP + 2 * SHANTY_POP + (hasShanty ? 5 : 0);
  world.total_time = 14;
  const auto count = world.map.constructionCount();

  // With an existing shanty the third draw chooses its cluster over the market.
  unsigned seed = 0;
  if(hasShanty) {
    for(; seed < 100; ++seed) {
      std::srand(seed);
      std::rand();
      std::rand();
      if(std::rand() % 8 != 0) break;
    }
    require(seed < 100, "could not select the existing shanty as the anchor");
  }
  std::srand(seed);
  world.do_time_step();
  require(world.total_time == 15 && world.map.constructionCount() == count &&
          world.stats.groupCount[GROUP_SHANTY] == hasShanty &&
          world.map(market)->getGroup() == GROUP_MARKET,
          "a full city did not advance one day with its existing buildings");
  require(world.people_pool == SHANTY_MIN_PP + 2 * SHANTY_POP,
          "unsuccessful shanty expansion altered the expected population loss");
}
}

void verifyShantyAtMapEdge() {
  World world(8);
  const MapPoint market(1, 1), available(5, 5), water(4, 5);
  marketConstructionGroup.placeItem(world, market);
  for(auto& tile : world.map) {
    if(world.map.is_visible(tile.point) && tile.is_bare() &&
       (tile.point.x < available.x || tile.point.y < available.y))
      tile.setTerrain(GROUP_WATER);
  }

  world.people_pool = SHANTY_MIN_PP + SHANTY_POP;
  std::ostringstream choices;
  choices << "C 7\nC 7\nP 1 " << available.x << ' ' << available.y << ' ' << GROUP_SHANTY << '\n';
  replayChoices(choices.str(), [&] { update_shanty(world); });
  const auto* shanty = world.map(available)->getConstruction();
  require(shanty && shanty->getConstructionGroup() == &shantyConstructionGroup &&
          world.map(available.se())->getReportingConstruction() == shanty &&
          world.map(available.e(2))->is_bare() && world.map(water)->is_water() &&
          world.stats.groupCount[GROUP_SHANTY] == 1,
          "settlement did not fit the only vacant plot beside the map border");
}

void verifyShantyMissingAnchor(bool growing) {
  World world(32);
  const MapPoint site(12, 12);
  marketConstructionGroup.placeItem(world, site);
  const auto* market = world.map(site)->getConstruction();
  // Exercise stale bookkeeping without putting an invalid construction on the map.
  world.stats.groupCount[GROUP_SHANTY] = growing ? 1 : 2;
  world.people_pool = growing ? SHANTY_MIN_PP + 2 * SHANTY_POP + 5 : 0;
  replayChoices(growing ? "C 12\nC 12\nC 1\n" : "C 12\nC 12\n",
                [&] { update_shanty(world); });
  require(world.map(site)->getConstruction() == market && world.map.constructionCount() == 1 &&
          world.stats.groupCount[GROUP_FIRE] == 0 && !world.popMessage(),
          "missing shanty anchor disturbed another building or produced a fire");
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
    verifyResidenceMigration();
    verifyResidencePollution();
    verifyResidenceDensity();
    verifyResidenceFire();
    verifyFireLifetime(false);
    verifyFireLifetime(true);
    verifyFireSpreadBarriers();
    verifyUniversityFunding();
    verifyMonthlyInsolvency(-2000000000);
    verifyMonthlyInsolvency(UNNAT_DEATHS_COST);
    verifyAnnualInterest();
    verifyMarketClosedTrade();
    verifyMarketWasteFire(false);
    verifyMarketWasteFire(true);
    verifyOverlappingCoverage();
    verifyAnnualFireCoverageAfterBuildingLoss();
    verifyShantyWaste();
    verifyShantyBulldoze();
    verifyShantyPopulation();
    verifyShantyClusterGrowth();
    verifyShantyOnFullMap(false);
    verifyShantyOnFullMap(true);
    verifyShantyAtMapEdge();
    verifyShantyMissingAnchor(true);
    verifyShantyMissingAnchor(false);
  } catch(const Message::Exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
  return 0;
}
