// Headless regression cases for player commands and their preview rules.
#include "lincity/UserOperation.hpp"
#include "lincity/modules/all_modules.hpp"
#include "lincity/world.hpp"
#include "util/simulation_trace.hpp"

#include <deque>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <type_traits>
#include <utility>

#define CHECK(expression) do { if(!(expression)) throw std::runtime_error(#expression); } while(false)

namespace {
static_assert(std::is_same_v<decltype(std::declval<const World&>().map(MapPoint{})), const MapTile*>);
static_assert(std::is_same_v<decltype(std::declval<const MapTile&>().getConstruction()), const Construction*>);
static_assert(std::is_same_v<decltype(std::declval<MapTile&>().getConstruction()), Construction*>);
static_assert(std::is_same_v<decltype(std::declval<const MapTile&>().getReportingConstruction()), const Construction*>);
static_assert(std::is_same_v<decltype(std::declval<MapTile&>().getReportingConstruction()), Construction*>);
static_assert(std::is_same_v<decltype(std::declval<const MapTile&>().getFrames()), const std::list<ExtraFrame>*>);
static_assert(std::is_same_v<decltype(std::declval<const Construction&>().getFrame()), const ExtraFrame&>);
static_assert(std::is_same_v<decltype(std::declval<const Construction&>().getConstructionGroup()), const ConstructionGroup*>);

const MapPoint site(8, 8);

template<class Error>
void checkRejected(const UserOperation& op, World& world, MapPoint point) {
  try {
    op.execute(world, point);
  } catch(const Message::Exception& e) {
    CHECK(dynamic_message_cast<Error>(e.getMessage()));
    return;
  }
  throw std::runtime_error("command unexpectedly succeeded");
}

void queryAndBoundaries() {
  World world(32);
  UserOperation query;
  Message::ptr why;
  CHECK(query.isAllowedHere(world, site, why));
  CHECK(query.cost(world, site) == 0);
  query.execute(world, site);
  CHECK(!world.isUpdated(World::Updatable::MAP));
  for(auto action : {UserOperation::ACTION_QUERY, UserOperation::ACTION_BUILD,
      UserOperation::ACTION_BULLDOZE, UserOperation::ACTION_EVACUATE,
      UserOperation::ACTION_FLOOD}) {
    UserOperation op{action, &marketConstructionGroup};
    CHECK(!op.isAllowedHere(world, site.w(8), why));
    auto outside = dynamic_message_cast<OutsideMapMessage>(why);
    CHECK(outside && outside->getPoint() == site.w(8));
    checkRejected<OutsideMapMessage>(op, world, site.w(8));
  }
}

void commandEntryPoints() {
  World world(32);
  const MapPoint border(0, 8);
  Message::ptr reason;
  for(auto action : {UserOperation::ACTION_QUERY, UserOperation::ACTION_BULLDOZE,
                     UserOperation::ACTION_EVACUATE, UserOperation::ACTION_FLOOD}) {
    UserOperation command{action};
    CHECK(command.isAllowed(world, reason) && !reason);
  }
  for(auto execute : {&World::bulldozeArea, &World::evacuateArea, &World::floodArea}) {
    bool rejected = false;
    try {
      (world.*execute)(border);
    } catch(const OutsideMapMessage::Exception& error) {
      rejected = error.getMessage()->getPoint() == border;
    }
    CHECK(rejected);
  }
  CHECK(!world.setMarketTrade(border, STUFF_FOOD, true, false));
  CHECK(!world.launchRocket(border));
  CHECK(!world.isUpdated(World::Updatable::MAP));

  world.total_money = 100000;
  const MapPoint edge(30, 30);
  UserOperation market{UserOperation::ACTION_BUILD, &marketConstructionGroup};
  CHECK(market.isAllowed(world, reason));
  CHECK(!market.isAllowedHere(world, edge, reason));
  const auto outOfMap = dynamic_message_cast<OutsideMapMessage>(reason);
  CHECK(outOfMap && outOfMap->getPoint() == edge);
  checkRejected<CannotBuildHereMessage>(market, world, edge);
  CHECK(world.total_money == 100000 && world.map.constructionCount() == 0);

  world.map(site)->setTerrain(GROUP_DESERT);
  UserOperation bulldoze{UserOperation::ACTION_BULLDOZE};
  CHECK(bulldoze.isAllowedHere(world, site, reason));
  CHECK(bulldoze.cost(world, site) == 0);
  bulldoze.execute(world, site);
  CHECK(world.total_money == 100000 && world.map(site)->group == GROUP_DESERT);
}

void moveOverlayFrame() {
  World world(32);
  auto& source = *world.map(site);
  auto& destination = *world.map(site.e());
  auto moving = source.createframe();
  moving->frame = 7;
  auto first = destination.createframe();
  first->frame = 3;
  source.moveFrameTo(destination, moving);
  CHECK(!source.getFrames());
  CHECK(destination.getFrames()->size() == 2);
  CHECK(destination.getFrames()->front().frame == 3);
  CHECK(&destination.getFrames()->back() == &*moving);
  destination.killframe(moving);
  CHECK(destination.getFrames()->size() == 1);
  destination.killframe(first);
  CHECK(!destination.getFrames());
}

void windSelection() {
  World world(32);
  for(ConstructionGroup* group : std::initializer_list<ConstructionGroup*>{
      &windmillConstructionGroup, &windpowerConstructionGroup}) {
    UserOperation build{UserOperation::ACTION_BUILD, group};
    world.tech_level = windpowerConstructionGroup.tech - 1;
    CHECK(build.actualConstructionGroup(world) == &windmillConstructionGroup);
    world.tech_level++;
    CHECK(build.actualConstructionGroup(world) == &windpowerConstructionGroup);
    CHECK(build.cost(world) == windpowerConstructionGroup.getCosts(world));
  }
}

void floodAndTransport() {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  Message::ptr why;
  UserOperation flood{UserOperation::ACTION_FLOOD};
  CHECK(flood.isAllowedHere(world, site, why));
  const int cost = flood.cost(world, site);
  flood.execute(world, site);
  CHECK(world.total_money == -cost);
  CHECK(world.map(site)->is_water());
  CHECK(!flood.isAllowedHere(world, site, why));
  CHECK(dynamic_message_cast<SpaceOccupiedMessage>(why));
  checkRejected<SpaceOccupiedMessage>(flood, world, site);
  CHECK(world.total_money == -cost);

  for(auto [land, bridge] : {std::pair{&trackConstructionGroup, &trackbridgeConstructionGroup},
      {&roadConstructionGroup, &roadbridgeConstructionGroup},
      {&railConstructionGroup, &railbridgeConstructionGroup}}) {
    UserOperation build{UserOperation::ACTION_BUILD, land};
    CHECK(build.actualConstructionGroup(world, site) == bridge);
    CHECK(build.cost(world, site) == bridge->getCosts(world));
    build.constructionGroup = bridge;
    CHECK(build.actualConstructionGroup(world, site.n()) == land);
  }

  UserOperation track{UserOperation::ACTION_BUILD, &trackConstructionGroup};
  CHECK(track.isAllowedHere(world, site, why));
  const int bridgeCost = track.cost(world, site);
  track.execute(world, site);
  CHECK(world.map(site)->getGroup() == GROUP_TRACK_BRIDGE);
  CHECK(world.total_money == -cost - bridgeCost);
  CHECK(!track.isAllowedHere(world, site, why));
  checkRejected<CannotBuildHereMessage>(track, world, site);

  UserOperation evacuate{UserOperation::ACTION_EVACUATE};
  CHECK(!evacuate.isAllowedHere(world, site, why));
  CHECK(dynamic_message_cast<CannotEvacuateThisMessage>(why));
  checkRejected<CannotEvacuateThisMessage>(evacuate, world, site);
  checkRejected<CannotEvacuateNothingMessage>(evacuate, world, site.n());
}

void floodRejectedAtCreditLimit() {
  World world(32);
  world.total_money = -2000000000;
  auto& tile = *world.map(site);
  const auto originalFlags = tile.flags;
  UserOperation flood{UserOperation::ACTION_FLOOD};
  checkRejected<OutOfMoneyMessage>(flood, world, site);
  CHECK(tile.group == GROUP_BARE && tile.flags == originalFlags);
  CHECK(world.total_money == -2000000000);
  CHECK(world.stats.expenses.construction.acc == 0);
  CHECK(!world.isUpdated(World::Updatable::MAP));
  CHECK(!world.isUpdated(World::Updatable::MONEY));
}

void marketCommands() {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  world.total_money = 1000000;
  Message::ptr why;
  UserOperation build{UserOperation::ACTION_BUILD, &marketConstructionGroup};
  CHECK(build.isAllowedHere(world, site, why));
  const int cost = build.cost(world, site);
  build.execute(world, site);
  CHECK(world.total_money == 1000000 - cost);
  auto* market = dynamic_cast<Market*>(world.map(site.se())->getReportingConstruction());
  CHECK(market);
  const int capacity = market->commodityRuleCount[STUFF_FOOD].maxload;
  CHECK(world.setMarketTrade(site.se(), STUFF_FOOD, true, true));
  CHECK(market->commodityRuleCount[STUFF_FOOD].take && market->commodityRuleCount[STUFF_FOOD].give);
  CHECK(world.setMarketTrade(site, STUFF_FOOD, false, false));
  CHECK(!market->commodityRuleCount[STUFF_FOOD].take && !market->commodityRuleCount[STUFF_FOOD].give);
  CHECK(market->commodityRuleCount[STUFF_FOOD].maxload == capacity);
  CHECK(!world.setMarketTrade(site, STUFF_HIVOLT, true, true));
  CHECK(!world.setPortTrade(site, STUFF_FOOD, false, true));

  UserOperation evacuate{UserOperation::ACTION_EVACUATE};
  CHECK(evacuate.isAllowedHere(world, site.se(), why));
  evacuate.execute(world, site.se());
  CHECK(market->flags & FLAG_EVACUATE);
  CHECK(!market->commodityRuleCount[STUFF_FOOD].take && market->commodityRuleCount[STUFF_FOOD].give);
  evacuate.execute(world, site);
  CHECK(!(market->flags & FLAG_EVACUATE));
  CHECK(market->commodityRuleCount[STUFF_FOOD].take && market->commodityRuleCount[STUFF_FOOD].give);
  CHECK(world.total_money == 1000000 - cost);

  UserOperation bulldoze{UserOperation::ACTION_BULLDOZE};
  CHECK(bulldoze.isAllowedHere(world, site.se(), why));
  CHECK(bulldoze.cost(world, site) == marketConstructionGroup.bul_cost);
  CHECK(bulldoze.cost(world, site.se()) == marketConstructionGroup.bul_cost);
  bulldoze.execute(world, site.se());
  CHECK(world.total_money == 1000000 - cost - marketConstructionGroup.bul_cost);
  CHECK(!world.map(site.se())->getReportingConstruction());
  CHECK(!world.map(site)->getConstruction());
  world.do_time_step(); // remove the detached construction
}

void marketSupplyAndWaste() {
  World world(32);
  const auto supplier = site.e(5);
  marketConstructionGroup.placeItem(world, site);
  organic_farmConstructionGroup.placeItem(world, supplier);
  auto& market = *world.map(site)->getConstruction();
  auto& farm = *world.map(supplier)->getConstruction();
  farm.bootstrap_commodities(50);
  CHECK(farm.commodityCount[STUFF_FOOD] == 5500);
  CHECK(farm.commodityCount[STUFF_WASTE] == 0);
  CHECK(farm.tellstuff(STUFF_FOOD, -2) == 500);
  CHECK(farm.tellstuff(STUFF_FOOD, 499) == 500 && farm.tellstuff(STUFF_FOOD, 501) == -1);

  CHECK(world.setMarketTrade(site, STUFF_FOOD, true, false));
  market.update();
  CHECK(market.commodityCount[STUFF_FOOD] == 0 && farm.commodityCount[STUFF_FOOD] == 5500);

  CHECK(world.setMarketTrade(site, STUFF_FOOD, false, true));
  market.levelStuff(STUFF_LABOR, 1);
  market.update();
  CHECK(market.commodityCount[STUFF_FOOD] == 0 && farm.commodityCount[STUFF_FOOD] == 5500);
  CHECK(world.taxable.labor == 1);

  CHECK(world.setMarketTrade(site, STUFF_FOOD, true, false));
  market.levelStuff(STUFF_LABOR, 1);
  market.update();
  CHECK(market.commodityCount[STUFF_FOOD] == 5280 && farm.commodityCount[STUFF_FOOD] == 220);
  CHECK(world.taxable.labor == 2);

  world.total_time = 1;
  const int threshold = 85 * MAX_WASTE_IN_MARKET / 100;
  market.levelStuff(STUFF_WASTE, threshold - 1);
  market.update();
  CHECK(market.commodityCount[STUFF_WASTE] == threshold - 1);
  CHECK(world.map(site.se())->pollution == 0);
  market.levelStuff(STUFF_WASTE, threshold);
  std::istringstream choices("LINCITY-SIMULATION-TRACE 1\nS market.waste-fire 0\nZ\n");
  SimulationTrace trace(choices);
  market.update();
  trace.finish();
  CHECK(market.commodityCount[STUFF_WASTE] == threshold - 7 * MAX_WASTE_IN_MARKET / 10);
  CHECK(world.map(site.se())->pollution == MAX_WASTE_IN_MARKET / 20);
  CHECK(!world.popMessage() && world.map(site)->getGroup() == GROUP_MARKET);
}

void marketStaffingAndAnimation() {
  struct Density { int fill; int staff; const char* resource; };
  for(const auto& spec : {Density{0, 1, "MarketEmpty"}, Density{15, 5, "MarketLow"},
                          Density{35, 12, "MarketMed"}, Density{70, 28, "MarketFull"}}) {
    World world(32);
    world.total_time = 17;
    marketConstructionGroup.placeItem(world, site);
    auto& market = dynamic_cast<Market&>(*world.map(site)->getConstruction());
    market.bootstrap_commodities(spec.fill);
    market.update();
    market.animate(0);
    CHECK(market.labor == spec.staff);
    CHECK(market.getFrame().resourceGroup == ResourceGroup::resMap.at(spec.resource));
  }

  World world(32);
  marketConstructionGroup.placeItem(world, site);
  auto& market = *world.map(site)->getConstruction();
  world.total_time = 1;
  market.levelStuff(STUFF_WASTE, MAX_WASTE_IN_MARKET);
  std::istringstream choices("LINCITY-SIMULATION-TRACE 1\nS market.waste-fire 0\nC 175\nZ\n");
  SimulationTrace trace(choices);
  market.update();
  market.animate(100);
  trace.finish();
  const auto& fire = world.map(site)->getFrames()->back();
  CHECK(fire.resourceGroup == ResourceGroup::resMap.at("Fire") && fire.frame == 0);
  market.animate(100 + ANIM_THRESHOLD(6 * WASTE_BURN_TIME));
  CHECK(fire.frame == -1);
}

void evacuationAndNeighborhood() {
  World world(32);
  const auto farmSite = site.e(5);
  const auto roadSite = farmSite.e(GROUP_ORGANIC_FARM_SIZE);
  marketConstructionGroup.placeItem(world, site);
  organic_farmConstructionGroup.placeItem(world, farmSite);
  roadConstructionGroup.placeItem(world, roadSite);
  auto& market = *world.map(site)->getConstruction();
  auto& farm = *world.map(farmSite)->getConstruction();
  auto& road = *world.map(roadSite)->getConstruction();
  world.evacuateArea(farmSite);
  CHECK(farm.flags & FLAG_EVACUATE);
  farm.levelStuff(STUFF_FOOD, 1100);
  CHECK(farm.tellstuff(STUFF_FOOD, 500) == 1100);

  market.levelStuff(STUFF_LABOR, 1);
  market.update();
  CHECK(farm.commodityCount[STUFF_FOOD] == 0 && market.commodityCount[STUFF_FOOD] == 1100);
  CHECK(farm.tellstuff(STUFF_FOOD, 500) == -1);

  farm.levelStuff(STUFF_FOOD, 1300);
  road.trade();
  CHECK(farm.commodityCount[STUFF_FOOD] == 0 && road.commodityCount[STUFF_FOOD] == 1300);
  world.bulldozeArea(farmSite);
  farm.levelStuff(STUFF_FOOD, 1000);
  market.levelStuff(STUFF_LABOR, 1);
  market.update();
  road.trade();
  CHECK(farm.commodityCount[STUFF_FOOD] == 1000);
  CHECK(market.commodityCount[STUFF_FOOD] == 1100 && road.commodityCount[STUFF_FOOD] == 1300);
  world.do_time_step();
  CHECK(world.map.constructionCount() == 2);
}

void portCommands() {
  World world(32);
  world.total_money = 1000000;
  UserOperation build{UserOperation::ACTION_BUILD, &portConstructionGroup};
  Message::ptr why;
  world.tech_level = portConstructionGroup.tech;
  CHECK(!build.isAllowedHere(world, site, why));
  CHECK(dynamic_message_cast<PortRequiresRiverMessage>(why));
  for(int i = 0; i < portConstructionGroup.size; ++i) {
    auto* tile = world.map(site.e(portConstructionGroup.size).s(i));
    tile->setTerrain(GROUP_WATER);
    tile->flags |= FLAG_IS_RIVER;
  }
  CHECK(build.isAllowedHere(world, site, why));
  build.execute(world, site);
  auto* port = dynamic_cast<Port*>(world.map(site)->getConstruction());
  CHECK(port);
  auto& rule = port->commodityRuleCount[STUFF_FOOD];
  CHECK(world.setPortTrade(site, STUFF_FOOD, true, false));
  CHECK(rule.take && !rule.give);
  CHECK(world.setPortTrade(site, STUFF_FOOD, false, true));
  CHECK(!rule.take && rule.give);
  CHECK(world.setPortTrade(site, STUFF_FOOD, true, true));
  CHECK(!rule.take && !rule.give);
  CHECK(!world.setPortTrade(site, STUFF_LABOR, true, false));
  CHECK(!world.setPortTrade(site, STUFF_COUNT, false, true));
}

void rocketCommands() {
  World world(32);
  UserOperation build{UserOperation::ACTION_BUILD, &rocketPadConstructionGroup};
  Message::ptr why;
  CHECK(!build.isAllowed(world, why));
  CHECK(dynamic_message_cast<NotEnoughTechMessage>(why));
  checkRejected<CannotBuildMessage>(build, world, site);
  world.tech_level = MAX_TECH_LEVEL;
  CHECK(!build.isAllowed(world, why));
  CHECK(dynamic_message_cast<OutOfMoneyMessage>(why));
  world.total_money = build.cost(world);
  build.execute(world, site);
  auto* rocket = dynamic_cast<RocketPad*>(world.map(site)->getConstruction());
  CHECK(rocket && world.total_money == 0);
  CHECK(!world.launchRocket(site));
  rocket->stage = RocketPad::AWAITING;
  CHECK(world.launchRocket(site));
  CHECK(rocket->stage == RocketPad::LAUNCHING);
  CHECK(!world.launchRocket(site));
}

void populationHistoryWindow() {
  World world(32);
  Stats& stats = world.stats;
  stats.history.pop = {30, 20};
  stats.history.ppool = {5};
  stats.history.nojobs = {7, 6, 4};
  stats.history.starve.clear();
  stats.ensurePopulationHistory(2);
  CHECK(stats.history.pop == std::deque<int>({30, 20}));
  CHECK(stats.history.ppool == std::deque<int>({5, 0}));
  CHECK(stats.history.nojobs == std::deque<int>({7, 6, 4}));
  CHECK(stats.history.starve == std::deque<int>({0, 0}));
  stats.ensurePopulationHistory(300);
  stats.population.population_m += 45;
  stats.monthly();
  CHECK(stats.history.pop.size() == 300 && stats.history.pop[0] == 45);
  CHECK(stats.history.pop[1] == 30 && stats.history.pop[2] == 20);
  CHECK(stats.history.ppool.size() == 300 && stats.history.ppool[1] == 5);
  CHECK(stats.history.nojobs.size() == 300 && stats.history.nojobs[1] == 7);
  CHECK(stats.history.starve.size() == 300 && stats.history.starve[1] == 0);
  CHECK(stats.history.tech.size() == 12);
}

void terrainAndSurvey() {
  World world(32);
  UserOperation bulldoze{UserOperation::ACTION_BULLDOZE};
  world.map(site)->setTerrain(GROUP_WATER);
  world.map(site)->flags |= FLAG_IS_RIVER;
  const int waterCost = bulldoze.cost(world, site);
  bulldoze.execute(world, site);
  CHECK(world.map(site)->group == GROUP_BARE);
  CHECK(!(world.map(site)->flags & FLAG_IS_RIVER));
  CHECK(world.total_money == -waterCost);
  const int landCost = bulldoze.cost(world, site);
  bulldoze.execute(world, site);
  CHECK(world.map(site)->group == GROUP_DESERT);
  bulldoze.execute(world, site);
  CHECK(world.total_money == -waterCost - landCost);
  const int beforeSurvey = world.total_money;
  world.do_coal_survey();
  CHECK(world.coal_survey_done);
  world.do_coal_survey();
  CHECK(world.total_money == beforeSurvey - 1000000);
  CHECK(world.stats.expenses.coalSurvey.acc == 1000000);
}

void protectedMonument() {
  World world(32);
  monumentConstructionGroup.placeItem(world, site);
  Message::ptr why;
  UserOperation bulldoze{UserOperation::ACTION_BULLDOZE};
  CHECK(!bulldoze.isAllowedHere(world, site, why));
  CHECK(dynamic_message_cast<CannotBulldozeIncompleteMonumentMessage>(why));
  checkRejected<CannotBulldozeIncompleteMonumentMessage>(bulldoze, world, site);
  CHECK(world.map(site)->getConstruction() && world.total_money == 0);
}
} // namespace

int main() {
  initializeModules();
  try {
    queryAndBoundaries();
    commandEntryPoints();
    windSelection();
    moveOverlayFrame();
    floodAndTransport();
    floodRejectedAtCreditLimit();
    marketCommands();
    marketSupplyAndWaste();
    marketStaffingAndAnimation();
    evacuationAndNeighborhood();
    portCommands();
    rocketCommands();
    protectedMonument();
    terrainAndSurvey();
    populationHistoryWindow();
  } catch(const Message::Exception& e) {
    std::cerr << e.what() << '\n';
    return 1;
  } catch(const std::exception& e) {
    std::cerr << e.what() << '\n';
    return 1;
  }
}
