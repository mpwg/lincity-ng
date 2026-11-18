#include "lincity/Vehicles.hpp"
#include "lincity/modules/blacksmith.hpp"
#include "lincity/modules/fire.hpp"
#include "lincity/modules/port.hpp"
#include "lincity/modules/track_road_rail.hpp"
#include "lincity/modules/windmill.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"

#include <algorithm>
#include <climits>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>

namespace {
namespace fs = std::filesystem;
const MapPoint site(12, 12);

#define CHECK(expression) do { if(!(expression)) throw std::runtime_error( \
  std::string(__func__) + " line " + std::to_string(__LINE__) + ": " #expression); } while(false)

void port() {
  World world(32);
  portConstructionGroup.placeItem(world, site);
  auto& building = dynamic_cast<Port&>(*world.map(site)->getConstruction());
  world.total_money = 500;
  world.total_time = 97;
  building.levelStuff(STUFF_GOODS, 999);
  building.levelStuff(STUFF_LABOR, 300);
  CHECK(world.setPortTrade(site, STUFF_GOODS, false, true));
  CHECK(world.setPortTrade(site, STUFF_FOOD, true, false));
  world.do_time_step();
  CHECK(building.commodityCount[STUFF_FOOD] == 5000);
  CHECK(building.commodityCount[STUFF_GOODS] == 500);
  CHECK(building.commodityCount[STUFF_LABOR] == 200);
  CHECK(world.total_money == 450 && world.stats.expenses.import.acc == 50);
  CHECK(world.taxable.trade_ex == 14 && building.pence == 97);
  CHECK(world.tech_level == 2 && world.map(site)->pollution == 1);

  world.do_time_step();
  CHECK(building.commodityCount[STUFF_FOOD] == 7500);
  CHECK(building.commodityCount[STUFF_GOODS] == 250);
  CHECK(building.commodityCount[STUFF_LABOR] == 100);
  CHECK(world.total_money == 425 && world.stats.expenses.import.acc == 75);
  CHECK(world.taxable.trade_ex == 22 && building.pence == 47);
  CHECK(building.lastm_ic == 7500 && building.lastm_et == 2247 && building.busy == 2);

  world.tradeRule[STUFF_FOOD].take = false; // export quantity is now below threshold
  world.do_time_step();
  CHECK(building.commodityCount[STUFF_LABOR] == 100);
  CHECK(world.total_money == 425 && world.taxable.trade_ex == 22 && building.pence == 47);
}

void checkAccountingLimits() {
  World world(32);
  auto& expenses = world.stats.expenses.construction;
  auto& income = world.stats.income.export_tax;
  world.total_money = 3;
  world.expense(3, expenses, false);
  auto warning = dynamic_message_cast<OutOfMoneyMessage>(world.popMessage());
  CHECK(warning && !warning->isOutOfCredit() && !world.popMessage());
  world.expense(2, expenses);
  CHECK(world.total_money == -2 && expenses.acc == 5 && !world.popMessage());

  for(bool allowCredit : {false, true}) {
    world.clearUpdated(World::Updatable::MONEY);
    bool rejected = false;
    try { world.expense(INT_MAX, expenses, allowCredit); }
    catch(const OutOfMoneyMessage::Exception& ex) {
      rejected = ex.getMessage()->isOutOfCredit() == allowCredit;
    }
    CHECK(rejected && world.total_money == -2 && expenses.acc == 5);
    CHECK(!world.isUpdated(World::Updatable::MONEY) && !world.popMessage());
  }
  world.total_money = INT_MAX - 1;
  income.acc = INT_MIN + 1;
  world.income(7, income);
  CHECK(world.total_money == INT_MAX && income.acc == INT_MIN);
  expenses.acc = INT_MAX - 1;
  world.expense(8, expenses);
  CHECK(world.total_money == INT_MAX - 8 && expenses.acc == INT_MAX);
}

void calendar() {
  checkAccountingLimits();
  World world(32);
  world.total_time = 1197;
  world.people_pool = 7;
  world.total_money = 1000;
  world.taxable.labor = 500;
  world.taxable.coal = 10;
  world.taxable.goods = 200;
  world.taxable.trade_ex = 26000;
  world.stats.population.housed_m += 150;
  world.stats.population.population_m += 150;
  world.stats.population.unemployed_m += 24;
  world.stats.population.unnat_deaths_m += 1;
  world.money_rates.dole = 25;
  world.do_time_step();
  CHECK(world.total_money == 1000 && world.stats.population.population_m.acc == 150);
  world.do_time_step();
  CHECK(world.total_time == 1199);
  CHECK(world.stats.expenses.deaths.acc == 500 && world.stats.expenses.unemployment.acc == 6);
  CHECK(world.stats.income.export_tax.acc == -25900 && world.stats.income.income_tax.acc == -40);
  CHECK(world.stats.income.coal_tax.acc == -15 && world.stats.income.goods_tax.acc == -2);
  CHECK(world.taxable.labor == 0 && world.taxable.goods == 0 &&
        world.taxable.coal == 0 && world.taxable.trade_ex == 0);
  CHECK(world.total_money == 26451 && world.stats.total_money == 494);
  CHECK(world.stats.population.population_m.acc == 850);
  world.do_time_step();
  CHECK(world.total_time == 1200 && world.total_money == 26451 && world.people_pool == 7);
  CHECK(world.stats.history.money.front() == 494 && world.stats.history.pop.front() == 850);
  CHECK(world.stats.history.ppool.front() == 700 && world.stats.history.nojobs.front() == 24);
  CHECK(world.stats.population.population_m.stat == 850 && world.stats.population.population_m.acc == 0);
  CHECK(world.stats.income.export_tax.stat == -25900 && world.stats.income.export_tax.acc == 0);
  CHECK(world.stats.expenses.deaths.stat == 500 && world.stats.expenses.deaths.acc == 0);
}

void windmill(const fs::path& saved) {
  World world(32);
  world.total_money = 10;
  const MapPoint other(20, 20);
  for(auto p : {site, other}) {
    windmillConstructionGroup.placeItem(world, p);
    world.map(p)->getConstruction()->levelStuff(STUFF_LABOR, 50);
  }
  // The two mills share the initial payment, then each pays on day four.
  const int balance[] = {9, 9, 9, 7, 7};
  for(int day = 1; day <= 5; ++day) {
    world.do_time_step();
    CHECK(world.total_money == balance[day-1]);
    CHECK(world.stats.expenses.windmill.acc == 10 - balance[day-1]);
    for(auto p : {site, other}) {
      auto& mill = *world.map(p)->getConstruction();
      CHECK(mill.commodityCount[STUFF_LABOR] == 50 - day * 10);
      CHECK(mill.commodityCount[STUFF_LOVOLT] == day * 450);
    }
  }

  // New worlds and freshly loaded games start their own payment cycle.
  World second(32);
  second.total_money = 10;
  windmillConstructionGroup.placeItem(second, site);
  second.do_time_step();
  CHECK(second.total_money == 9 && second.stats.expenses.windmill.acc == 1);
  second.save(saved);
  auto loaded = World::load(saved);
  loaded->do_time_step();
  CHECK(loaded->total_money == 8);
  second.do_time_step();
  CHECK(second.total_money == 9);

  // At the credit limit a payment fails; a subsequent tick can retry it.
  auto& mill = *second.map(site)->getConstruction();
  mill.levelStuff(STUFF_LABOR, 20);
  second.total_time = 3;
  second.total_money = -2000000000;
  second.do_time_step();
  CHECK(second.total_money == -2000000000 && second.stats.expenses.windmill.acc == 1);
  CHECK(mill.commodityCount[STUFF_LABOR] == 20 && mill.commodityCount[STUFF_LOVOLT] == 0);
  second.total_money += 1;
  second.do_time_step();
  CHECK(second.total_money == -2000000000 && second.stats.expenses.windmill.acc == 2);
  CHECK(mill.commodityCount[STUFF_LABOR] == 10 && mill.commodityCount[STUFF_LOVOLT] == 450);
}

int carFrames(const World& world) {
  int count = 0;
  const auto* bluecar = ResourceGroup::resMap.at("Bluecar");
  for(const auto& tile : world.map) {
    if(auto frames = tile.getFrames())
      for(const auto& frame : *frames)
        if(frame.resourceGroup == bluecar) ++count;
  }
  return count;
}

void vehicles() {
  for(auto strategy : {VEHICLE_STRATEGY_MAXIMIZE, VEHICLE_STRATEGY_MINIMIZE}) {
    World world(32);
    const auto start = site.w();
    for(auto p : {start, site, site.n(), site.s()})
      roadConstructionGroup.placeItem(world, p);
    world.map(start)->getConstruction()->levelStuff(STUFF_LABOR, 100);
    world.map(site)->getConstruction()->levelStuff(STUFF_LABOR,
      strategy == VEHICLE_STRATEGY_MAXIMIZE ? 150 : 50);
    world.map(site.n())->getConstruction()->levelStuff(STUFF_LABOR, 200);
    auto* car = new Vehicle(world, start, VEHICLE_BLUECAR, strategy);
    car->update(1);
    CHECK(carFrames(world) == 1);
    CHECK(car->point == site && car->prev == start);
    car->update(52);
    CHECK(carFrames(world) == 1);
    CHECK(car->point == (strategy == VEHICLE_STRATEGY_MAXIMIZE ? site.n() : site.s()));
    CHECK(car->direction == (strategy == VEHICLE_STRATEGY_MAXIMIZE ? 3 : 1));
    car->update(3000); // no forward route; update removes the vehicle and its frame
    CHECK(carFrames(world) == 0);
  }
}

void fire(const fs::path& saved) {
  World world(32);
  fireConstructionGroup.placeItem(world, site);
  const auto target = site.e();
  blacksmithConstructionGroup.placeItem(world, target);
  auto& source = dynamic_cast<Fire&>(*world.map(site)->getConstruction());
  source.burning_days = 37;
  source.days_before_spread = 1;
  // Find a reproducible eastward spread event using this runtime's rand().
  unsigned seed = 0;
  for(; seed < 100000; ++seed) {
    std::srand(seed);
    if(std::rand() % 80 == 2 && std::rand() % 100 < 60) break;
  }
  CHECK(seed < 100000);
  std::srand(seed);
  source.update();
  CHECK(source.burning_days == 38 && source.days_before_spread == 150);
  for(auto p : {target, target.e(), target.s(), target.se()})
    CHECK(world.map(p)->getGroup() == GROUP_FIRE);
  auto& smoke = dynamic_cast<Fire&>(*world.map(target)->getConstruction());
  smoke.burning_days = 6001;
  smoke.smoking_days = 123;
  world.do_time_step(); // prune the detached smithy before saving
  world.save(saved);
  auto loaded = World::load(saved);
  auto& burning = dynamic_cast<Fire&>(*loaded->map(site)->getConstruction());
  auto& smoking = dynamic_cast<Fire&>(*loaded->map(target)->getConstruction());
  CHECK(loaded->map.constructionCount() == 5);
  CHECK(burning.burning_days == 39 && burning.days_before_spread == 149);
  CHECK(smoking.burning_days == 6001 && smoking.smoking_days == 124);
  loaded->do_time_step();
  CHECK(burning.burning_days == 40 && burning.days_before_spread == 148);
  CHECK(smoking.smoking_days == 125);
}

void checkSmithy(World& world, int day) {
  const auto& smith = dynamic_cast<const Blacksmith&>(*world.map(site)->getConstruction());
  const int batches = std::min(day - 96, 5);
  CHECK(world.total_time == day && world.total_money == 1703 && world.people_pool == 7);
  CHECK(world.tech_level == (day < 99 ? 41 : 43));
  CHECK(world.map.recentPoint == MapPoint(9, 15) && world.map.len() == 32);
  CHECK(world.map.constructionCount() == 1 && world.map(site.se())->getReportingConstruction() == &smith);
  CHECK(smith.commodityCount[STUFF_LABOR] == 175 - 35 * batches);
  CHECK(smith.commodityCount[STUFF_COAL] == 5 - batches && smith.commodityCount[STUFF_STEEL] == 5 - batches);
  CHECK(smith.commodityCount[STUFF_GOODS] == 7 + 50 * batches);
  CHECK(world.stats.population.population_m.stat == (day >= 100 ? 700 : 0));
}

void saves(const fs::path& fixtures, const fs::path& saved) {
  auto direct = World::load(fixtures / "blacksmith-2140.xml");
  CHECK(direct->given_scene == "Reference smithy");
  const auto& tile = *direct->map(MapPoint(6, 8));
  CHECK(tile.group == GROUP_WATER && tile.ore_reserve == 37 &&
        tile.ground.altitude == 14 && tile.ground.water_alt == 17);
  checkSmithy(*direct, 96);
  direct->do_time_step();
  checkSmithy(*direct, 97);
  direct->save(saved);
  auto restored = World::load(saved);
  for(int day = 97; day <= 103; ++day) {
    if(day > 97) {
      // Supply both external RNGs with the same inputs to both continuations.
      for(auto* world : {direct.get(), restored.get()}) {
        std::srand(615 + day);
        BasicUrbg::get().seed(615 + day);
        world->do_animate(day * 1000);
        world->do_time_step();
      }
    }
    checkSmithy(*direct, day);
    checkSmithy(*restored, day);
  }

  std::ofstream(saved)
    << "<lc-game ldsv-version='2140'><map size='32'>"
       "<Construction group='27' map-x='12' map-y='12'/>"
       "</map></lc-game>";
  CHECK(World::load(saved)->map.recentPoint == site);

  struct Invalid { const char* xml; const char* diagnostic; };
  for(const Invalid& invalid : {
    Invalid{"<lc-game/>", "failed to parse load/save version"},
    Invalid{"<lc-game ldsv-version='2129'/>", "version too old"},
    Invalid{"<lc-game ldsv-version='2141'/>", "version too new"},
    Invalid{"<lc-game ldsv-version='2140'><map size='32'><MapTile group='0' map-x='32' map-y='1'/></map></lc-game>", "outside the map"},
    Invalid{"<lc-game ldsv-version='2140'><map size='32'><Construction group='999' map-x='8' map-y='8'/></map></lc-game>", "invalid group"}
  }) {
    std::ofstream(saved) << invalid.xml;
    bool rejected = false;
    try { World::load(saved); }
    catch(const std::runtime_error& ex) {
      rejected = std::string(ex.what()).find(invalid.diagnostic) != std::string::npos;
    }
    CHECK(rejected);
  }
}
}

int main(int argc, char** argv) {
  if(argc != 4) return 2;
  std::srand(615);
  BasicUrbg::get().seed(615);
  const std::string name = argv[1];
  try {
    if(name == "port") port();
    else if(name == "calendar") calendar();
    else if(name == "windmill") windmill(argv[3]);
    else if(name == "vehicles") vehicles();
    else if(name == "fire") fire(argv[3]);
    else if(name == "saves") saves(argv[2], argv[3]);
    else return 2;
    if(name == "saves" || name == "fire" || name == "windmill") fs::remove(argv[3]);
  } catch(const std::exception& ex) {
    std::cerr << ex.what() << '\n';
    return 1;
  }
}
