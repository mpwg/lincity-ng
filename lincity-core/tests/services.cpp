#include "lincity/all_buildings.hpp"
#include "lincity/messages.hpp"
#include "lincity/modules/cricket.hpp"
#include "lincity/modules/firestation.hpp"
#include "lincity/modules/health_centre.hpp"
#include "lincity/modules/school.hpp"
#include "lincity/modules/university.hpp"
#include "lincity/modules/waterwell.hpp"
#include "lincity/world.hpp"

#include <iostream>
#include <stdexcept>

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

struct Recipe {
  int labor;
  int goods;
  int wasteCapacity;
  int runningCost;
  int techMade;
  Stat<int>& account;
};

void verifyRecipe(World& world, Construction& building, const Recipe& recipe,
                  bool chargesTreasury = true) {
  const int initialTech = world.tech_level;
  const int initialMoney = world.total_money;
  const int initialExpense = recipe.account.acc;
  const int waste = recipe.goods / 3;
  const int capacity = recipe.wasteCapacity;
  const auto tick = [&](int day, int labor, int goods, int stock, bool works) {
    building.levelStuff(STUFF_LABOR, labor);
    building.levelStuff(STUFF_GOODS, goods);
    building.levelStuff(STUFF_WASTE, stock);
    building.update();
    require(building.commodityCount[STUFF_LABOR] == labor - (works ? recipe.labor : 0),
            "unexpected daily labor consumption");
    require(building.commodityCount[STUFF_GOODS] == goods - (works ? recipe.goods : 0),
            "unexpected daily goods consumption");
    require(building.commodityCount[STUFF_WASTE] == stock + (works ? waste : 0),
            "unexpected daily waste production");
    require(world.tech_level == initialTech + (works ? recipe.techMade : 0),
            "unexpected daily technology production");
    require(recipe.account.acc == initialExpense + day * recipe.runningCost,
            "unexpected expenses on an operating or idle day");
    require(world.total_money == initialMoney - (chargesTreasury ? day * recipe.runningCost : 0),
            "unexpected treasury change");
  };

  tick(1, recipe.labor - 1, recipe.goods, capacity - waste, false);
  tick(2, recipe.labor, recipe.goods - 1, capacity - waste, false);
  tick(3, recipe.labor, recipe.goods, capacity - waste + 1, false);
  tick(4, recipe.labor, recipe.goods, capacity - waste, true);
}

template<typename Building>
void verifyCoverage(World& world, ConstructionGroup& group, const Recipe& recipe,
                    unsigned int coverage, unsigned int pendingCoverage) {
  const MapPoint site(8, 8);
  auto& building = place<Building>(world, group, site);
  world.total_time = 60;
  verifyRecipe(world, building, recipe);
  require(building.working_days == 1, "only one day had enough supplies to operate");

  // The two idle days before and during renewal bring the total to five.
  const MapPoint nearBorder(1, 1);
  const MapPoint border(0, 1);
  const MapPoint nearby(site.e(group.range - 1).s(group.range - 1));
  const MapPoint outside(site.e(group.range).s(group.range - 1));
  world.total_time = 73;
  world.do_time_step();
  require(!building.active && !(world.map(nearby)->flags & (coverage | pendingCoverage)),
          "coverage activated before day 75");

  world.do_time_step();
  require(building.active, "coverage should tolerate exactly five idle days");
  require(world.map(nearBorder)->flags & coverage, "coverage did not extend to the interior edge");
  require(!(world.map(border)->flags & coverage), "coverage reached the map border");
  require(world.map(nearby)->flags & coverage, "coverage omitted the last in-range tile");
  require(!(world.map(nearby)->flags & pendingCoverage), "day 75 did not commit pending coverage");
  require(!(world.map(outside)->flags & coverage), "coverage exceeded its range");

  world.total_time = 98;
  world.do_time_step();
  require(building.busy == 1 && building.working_days == 0,
          "monthly busy count did not record the one operating day");
  world.total_time = DAYS_BETWEEN_COVER + 74;
  world.do_time_step();
  require(!building.active && !(world.map(nearby)->flags & coverage),
          "coverage survived a missed service interval");

  building.levelStuff(STUFF_LABOR, recipe.labor);
  building.levelStuff(STUFF_GOODS, recipe.goods);
  building.levelStuff(STUFF_WASTE, 0);
  world.total_money = -2000000000;
  const int expenseBefore = recipe.account.acc;
  building.update();
  require(world.total_money == -2000000000 && recipe.account.acc == expenseBefore &&
          building.commodityCount[STUFF_LABOR] == recipe.labor &&
          building.commodityCount[STUFF_GOODS] == recipe.goods &&
          building.commodityCount[STUFF_WASTE] == 0,
          "service operated despite reaching the credit limit");
}

void verifyCoverageServices() {
  {
    World world(32);
    world.tech_level = MAX_TECH_LEVEL;
    verifyCoverage<HealthCentre>(world, healthCentreConstructionGroup,
      {HEALTH_CENTRE_LABOR, HEALTH_CENTRE_GOODS, MAX_WASTE_AT_HEALTH_CENTRE,
       HEALTH_RUNNING_COST * (1 + HEALTH_RUNNING_COST_MUL), 0, world.stats.expenses.health},
      FLAG_HEALTH_COVER, FLAG_HEALTH_COVER_CHECK);
  }
  {
    World world(32);
    world.tech_level = MAX_TECH_LEVEL / 2;
    verifyCoverage<FireStation>(world, fireStationConstructionGroup,
      {FIRESTATION_LABOR, FIRESTATION_GOODS, MAX_WASTE_AT_FIRESTATION,
       FIRESTATION_RUNNING_COST * (1 + FIRESTATION_RUNNING_COST_MUL / 2),
       0, world.stats.expenses.firestation}, FLAG_FIRE_COVER, FLAG_FIRE_COVER_CHECK);
  }
  {
    World world(32);
    world.tech_level = MAX_TECH_LEVEL;
    verifyCoverage<Cricket>(world, cricketConstructionGroup,
      {CRICKET_LABOR, CRICKET_GOODS, MAX_WASTE_AT_CRICKET, CRICKET_RUNNING_COST,
       0, world.stats.expenses.cricket}, FLAG_CRICKET_COVER, FLAG_CRICKET_COVER_CHECK);
  }
}

void verifyEducation() {
  World world(32);
  world.total_money = 1000000;
  world.tech_level = GROUP_UNIVERSITY_TECH;
  world.total_time = 98;
  Message::ptr message;

  for(int schools = 0; schools < 4; ++schools) {
    require(!universityConstructionGroup.can_build(world, message) &&
            dynamic_cast<const NotEnoughStudentsMessage*>(message.get()),
            "university should need four schools");
    world.buildConstruction(schoolConstructionGroup, MapPoint(2 + schools * 3, 2));
  }

  auto& school = dynamic_cast<School&>(*world.map(MapPoint(2, 2))->getConstruction());
  const Recipe schoolRecipe{LABOR_MAKE_TECH_SCHOOL, GOODS_MAKE_TECH_SCHOOL,
    MAX_WASTE_AT_SCHOOL, SCHOOL_RUNNING_COST, TECH_MADE_BY_SCHOOL, world.stats.expenses.school};
  verifyRecipe(world, school, schoolRecipe, false);
  require(school.total_tech_made == TECH_MADE_BY_SCHOOL, "school lost its teaching total");
  world.total_time = 99;
  school.update();
  require(school.working_days == 1 && school.busy == 0, "school closed its month on day 99");
  world.total_time = 100;
  school.update();
  require(school.working_days == 0 && school.busy == 1, "school did not close its month on day 100");

  const MapPoint campus(10, 10);
  world.buildConstruction(universityConstructionGroup, campus);
  require(!universityConstructionGroup.can_build(world, message) &&
          dynamic_cast<const NotEnoughStudentsMessage*>(message.get()),
          "the same four schools qualified a second university");
  auto& university = dynamic_cast<University&>(*world.map(campus)->getConstruction());
  world.total_time = 98;
  verifyRecipe(world, university, {UNIVERSITY_LABOR, UNIVERSITY_GOODS, MAX_WASTE_AT_UNIVERSITY,
    UNIVERSITY_RUNNING_COST, UNIVERSITY_TECH_MADE, world.stats.expenses.university});
  require(university.total_tech_made == UNIVERSITY_TECH_MADE, "university lost its research total");
  world.total_time = 99;
  university.update();
  require(university.working_days == 0 && university.busy == 1,
          "university did not close its month on day 99");
}

void verifyWaterwell() {
  World world(32);
  const MapPoint site(8, 8);
  Message::ptr message;
  world.map(site.e(2))->flags |= FLAG_HAS_UNDERGROUND_WATER;
  require(!waterwellConstructionGroup.can_build_here(world, site, message) &&
          dynamic_cast<const DesertHereMessage*>(message.get()),
          "a wet tile outside the footprint qualified a dry well");
  world.map(site.s().e())->flags |= FLAG_HAS_UNDERGROUND_WATER;
  require(waterwellConstructionGroup.can_build_here(world, site, message),
          "water below the far corner did not qualify the well");
  world.map(site.s())->flags |= FLAG_HAS_UNDERGROUND_WATER;
  world.buildConstruction(waterwellConstructionGroup, site);
  auto& well = dynamic_cast<Waterwell&>(*world.map(site)->getConstruction());
  const int output = 2 * WATER_PER_UGW;
  well.update();
  require(well.commodityCount[STUFF_WATER] == output,
          "well output should depend on the two wet tiles inside its footprint");
  well.levelStuff(STUFF_WATER, MAX_WATER_AT_WATERWELL - output + 1);
  well.update();
  require(well.commodityCount[STUFF_WATER] == MAX_WATER_AT_WATERWELL - output + 1,
          "well pumped with less than one day's room in its tank");
  well.levelStuff(STUFF_WATER, MAX_WATER_AT_WATERWELL - output);
  world.total_time = 99;
  well.update();
  require(well.commodityCount[STUFF_WATER] == MAX_WATER_AT_WATERWELL,
          "well did not pump when exactly one day's room was available");
  require(well.busy == 2 && well.working_days == 0, "well counted a blocked day as operating");
}
}

int main() {
  try {
    verifyCoverageServices();
    verifyEducation();
    verifyWaterwell();
  } catch(const Message::Exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
  return 0;
}
