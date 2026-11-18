#include "lincity/Vehicles.hpp"
#include "lincity/modules/all_modules.hpp"
#include "lincity/sustainable.hpp"
#include "lincity/transport.hpp"
#include "lincity/world.hpp"
#include "util/simulation_trace.hpp"

#include <array>
#include <iostream>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>

#define CHECK(test) do { if(!(test)) throw std::runtime_error(#test); } while(false)

namespace {
const MapPoint site(12, 12);

void terrainAndFootprints() {
  World world(32);
  struct Terrain { unsigned short id; const char* resource; };
  for(const auto& terrain : std::array<Terrain, 6>{{
    {GROUP_BARE, "Green"}, {GROUP_DESERT, "Desert"}, {GROUP_WATER, "Water"},
    {GROUP_TREE, "Tree"}, {GROUP_TREE2, "Tree2"}, {GROUP_TREE3, "Tree3"},
  }}) {
    auto& tile = *world.map(site);
    tile.setTerrain(terrain.id);
    tile.type = 3;
    CHECK(tile.getTileResourceGroup() == ResourceGroup::resMap.at(terrain.resource));
    CHECK(tile.getTileConstructionGroup()->group == terrain.id);
    CHECK(tile.getConstructionGroup() == tile.getTopConstructionGroup());
    CHECK(tile.getLowerstVisibleGroup() == terrain.id);
    CHECK(tile.getType() == 3 && tile.getTopType() == 3 && tile.getLowerstVisibleType() == 3);
    CHECK(tile.is_visible() && tile.is_bare() == (terrain.id != GROUP_WATER));
  }

  blacksmithConstructionGroup.placeItem(world, site);
  const auto& origin = *world.map(site);
  const auto& covered = *world.map(site.se());
  auto* building = world.map(site)->getConstruction();
  building->getFrame().frame = 4;
  CHECK(origin.getTopConstructionGroup() == &blacksmithConstructionGroup);
  CHECK(covered.getConstructionGroup() == &blacksmithConstructionGroup);
  CHECK(covered.getTopConstructionGroup() == covered.getTileConstructionGroup());
  CHECK(covered.getLowerstVisibleConstructionGroup() == &blacksmithConstructionGroup);
  CHECK(origin.getTopType() == 4 && covered.getType() == 4);
  CHECK(covered.getTopType() == covered.type && covered.getLowerstVisibleType() == 4);
  CHECK(origin.is_visible() && !covered.is_visible() && !covered.is_bare());
  building->flags |= FLAG_TRANSPARENT;
  CHECK(covered.getLowerstVisibleType() == covered.type);
  CHECK(covered.getLowerstVisibleConstructionGroup() == covered.getTileConstructionGroup());

  const auto dock = site.e(7);
  portConstructionGroup.placeItem(world, dock);
  CHECK(world.map(dock.e(2))->getLowerstVisibleGroup() == GROUP_PORT);
  CHECK(world.map(dock.e(3))->getLowerstVisibleGroup() == GROUP_WATER);
}

void mapSearchAndTerrain() {
  World world(32);
  auto& map = world.map;
  CHECK(map.cbegin()->point == MapPoint());
  CHECK(map.crbegin()->point == MapPoint(31, 31));
  CHECK(map.rbegin()->point == MapPoint(31, 31));
  CHECK(map.crend() - map.crbegin() == 1024);
  CHECK(map.cend() - map.cbegin() == 1024);
  CHECK(map.rend() - map.rbegin() == 1024);
  const Map& readOnlyMap = map;
  CHECK(readOnlyMap.rend() - readOnlyMap.rbegin() == 1024);
  CHECK(map.is_edge(MapPoint(1, 3)) && !map.is_border(MapPoint(1, 3)));
  CHECK(map.is_border(MapPoint(0, 3)) && !map.is_visible(MapPoint(0, 3)));
  CHECK(!map.find_group(site, GROUP_WATER));
  CHECK(map.find_bare_area(site, 2) == site);
  map(site)->setTerrain(GROUP_WATER);
  CHECK(map.find_group(site.e(2), GROUP_WATER) == site);
  CHECK(map.find_bare_area(site, 2) == site.ne());
  CHECK(!map.find_bare_area(site, 31));

  map(site)->ground.altitude = 5;
  CHECK(map.maximum(site) && !map.minimum(site));
  map(site.n())->ground.altitude = 10;
  CHECK(!map.maximum(site) && !map.minimum(site));
  map(site.s())->ground.altitude = 10;
  CHECK(map.saddlepoint(site));
  map(site)->ground.altitude = -1;
  CHECK(map.minimum(site) && !map.saddlepoint(site));
  CHECK(!map.checkEdgeMin(site));
  for(const auto corner : {MapPoint(1, 1), MapPoint(1, 30), MapPoint(30, 1), MapPoint(30, 30),
                           MapPoint(15, 1), MapPoint(1, 15)}) {
    map(corner)->ground.altitude = -1;
    CHECK(map.checkEdgeMin(corner));
    map(corner)->ground.altitude = 1;
    CHECK(!map.checkEdgeMin(corner));
  }
}

void riversAndCables() {
  World world(32);
  auto& map = world.map;
  auto* water = map(site);
  water->setTerrain(GROUP_WATER);
  map(site.w())->setTerrain(GROUP_WATER);
  map(site.w())->flags |= FLAG_IS_RIVER;
  map(site.e())->setTerrain(GROUP_WATER);
  map(site.e(2).n())->setTerrain(GROUP_WATER);
  map.connect_rivers(site.x, site.y);
  CHECK(water->is_river() && map(site.e())->is_river());
  CHECK(map(site.e(2).n())->is_lake());
  world.bulldozeArea(site);
  CHECK(world.total_money == -waterConstructionGroup.bul_cost);
  CHECK(!water->is_water() && !water->is_river());

  const MapPoint edge(1, 4);
  map(edge)->setTerrain(GROUP_WATER);
  map(edge.e())->setTerrain(GROUP_WATER);
  map(edge.e())->flags |= FLAG_IS_RIVER;
  map(edge.w())->setTerrain(GROUP_WATER);
  map.connect_rivers(edge.x, edge.y);
  CHECK(map(edge.w())->is_river() && map(edge)->is_river());

  for(const auto direction : {MapPoint(0,-1), MapPoint(-1,0), MapPoint(1,0), MapPoint(0,1)}) {
    World grid(32);
    const auto point = [&](int distance) {
      return MapPoint(site.x + distance * direction.x, site.y + distance * direction.y);
    };
    auto& crossing = *grid.map(point(1));
    crossing.setTerrain(GROUP_WATER);
    powerlineConstructionGroup.placeItem(grid, site);
    powerlineConstructionGroup.placeItem(grid, point(2));
    grid.map.connect_transport(0, 0, 32, 32);
    const auto cable = direction.x ? FLAG_POWER_CABLES_90 : FLAG_POWER_CABLES_0;
    CHECK(crossing.flags & cable);
    grid.bulldozeArea(point(2));
    CHECK(!(crossing.flags & cable));
  }
}

void transportFrames() {
  struct Kind { ConstructionGroup& land; ConstructionGroup& bridge; bool longRamp; };
  for(const auto& kind : std::array<Kind, 3>{{
    {trackConstructionGroup, trackbridgeConstructionGroup, false},
    {roadConstructionGroup, roadbridgeConstructionGroup, true},
    {railConstructionGroup, railbridgeConstructionGroup, true}
  }}) {
    const std::array<MapPoint, 4> directions{MapPoint(0,-1), MapPoint(-1,0),
                                           MapPoint(0,1), MapPoint(1,0)};
    for(unsigned index = 0; index < directions.size(); ++index) {
      World world(32);
      const auto d = directions[index];
      const auto at = [&](int step) { return MapPoint(site.x + d.x * step, site.y + d.y * step); };
      world.map(at(1))->setTerrain(GROUP_WATER);
      kind.bridge.placeItem(world, at(1));
      kind.land.placeItem(world, at(0));
      kind.land.placeItem(world, at(-1));
      world.map.connect_transport(8, 8, 16, 16);
      CHECK(world.map(at(0))->getType() == 13 + index);
      CHECK(world.map(at(1))->getType() == (d.x ? 1 : 0));
      CHECK(world.map(at(1))->getTransportGroup() == kind.land.group);
      CHECK(world.map(at(1))->getLowerstVisibleGroup() == GROUP_WATER);
      if(kind.longRamp)
        CHECK(world.map(at(-1))->getType() == 17 + index);

      world.bulldozeArea(at(-1));
      world.map(at(-1))->setTerrain(GROUP_WATER);
      kind.bridge.placeItem(world, at(-1));
      world.map.connect_transport(8, 8, 16, 16);
      CHECK(world.map(at(0))->getType() == (d.x ? 12 : 11));
      CHECK(world.map(at(0))->getConstruction()->flags & FLAG_TRANSPARENT);
    }
  }

  for(auto* road : {&trackConstructionGroup, &roadConstructionGroup}) {
    World crossings(32);
    railConstructionGroup.placeItem(crossings, site);
    road->placeItem(crossings, site.e());
    road->placeItem(crossings, site.w());
    crossings.map.connect_transport(8, 8, 16, 16);
    const int offset = road == &trackConstructionGroup ? 0 : 2;
    CHECK(crossings.map(site)->getType() == 22 + offset);
    crossings.bulldozeArea(site.e());
    crossings.bulldozeArea(site.w());
    road->placeItem(crossings, site.n());
    road->placeItem(crossings, site.s());
    crossings.map.connect_transport(8, 8, 16, 16);
    CHECK(crossings.map(site)->getType() == 21 + offset);
  }
}

void vehicleCrossings() {
  struct Route { MapPoint step; int direction; };
  for(const auto& route : std::array<Route, 4>{{
    {MapPoint(0, 1), 0}, {MapPoint(1, 0), 2}, {MapPoint(0, -1), 4}, {MapPoint(-1, 0), 6}
  }}) {
    World world(32);
    const auto at = [&](int n) {
      return MapPoint(site.x + n * route.step.x, site.y + n * route.step.y);
    };
    for(int n = 0; n < 6; ++n) {
      if(n == 2)
        railConstructionGroup.placeItem(world, at(n));
      else
        trackConstructionGroup.placeItem(world, at(n));
    }
    auto car = std::make_unique<Vehicle>(world, site, VEHICLE_BLUECAR);
    car->update(1);
    CHECK(car->point == at(1));
    car->update(car->anim + 1);
    CHECK(car->point == at(2) && car->direction == route.direction);
    CHECK(car->frameIt->frame == route.direction);
    car->update(car->anim + 1);
    CHECK(car->point == at(3) && car->direction == route.direction);
  }

  struct Crossing { ConstructionGroup& land; ConstructionGroup& bridge; };
  for(const auto& kind : std::array<Crossing, 2>{{
    {trackConstructionGroup, trackbridgeConstructionGroup},
    {roadConstructionGroup, roadbridgeConstructionGroup}
  }}) {
    World world(32);
    for(int n = 0; n < 6; ++n) {
      if(n == 2) {
        world.map(site.e(n))->setTerrain(GROUP_WATER);
        kind.bridge.placeItem(world, site.e(n));
      } else {
        kind.land.placeItem(world, site.e(n));
      }
    }
    auto car = std::make_unique<Vehicle>(world, site, VEHICLE_BLUECAR);
    car->update(1);
    car->update(car->anim + 1);
    CHECK(car->point == site.e(2) && car->frameIt->frame == 9);
    car->update(car->anim + 1);
    CHECK(car->point == site.e(3) && car->frameIt->frame == 2);
    car->update(car->anim + 1);
    CHECK(car->point == site.e(4) && car->frameIt->frame == 13);
  }
}

void eventDiagnostics() {
  World world(32);
  const auto outside = OutsideMapMessage::create(site);
  try {
    outside->throwEx();
  } catch(const Message::Exception& error) {
    CHECK(Message::fromCurrentException() == outside);
    CHECK(std::string(error.what()) == "outside map: (12,12)");
  }

  try {
    throw std::runtime_error("resource unavailable");
  } catch(...) {
    CHECK(Message::fromCurrentException()->str().find("resource unavailable") != std::string::npos);
  }
  try {
    throw 2;
  } catch(...) {
    CHECK(Message::fromCurrentException()->str() == "unknown exception");
  }

  world.total_time = NUMOF_DAYS_IN_MONTH - 2;
  world.people_pool = 0;
  world.do_time_step();
  const auto noPeople = dynamic_message_cast<NoPeopleLeftMessage>(world.popMessage());
  CHECK(noPeople && noPeople->getTime() == NUMOF_DAYS_IN_MONTH - 1);
  CHECK(noPeople->str() == "everyone died");
  CHECK(world.gameEnd);

  blacksmithConstructionGroup.placeItem(world, site);
  world.map(site)->flags |= FLAG_FIRE_COVER;
  const auto ignite = [&] {
    std::istringstream input("LINCITY-SIMULATION-TRACE 1\nC 12\nC 12\nC 0\nZ\n");
    SimulationTrace trace(input);
    // Placement after ignition is an observable side effect. Record those
    // entries only for the unprotected case below.
    world.do_random_fire();
    trace.finish();
  };
  ignite();
  CHECK(!world.popMessage() && world.map(site)->getGroup() == GROUP_BLACKSMITH);

  world.map(site)->flags &= ~FLAG_FIRE_COVER;
  std::istringstream fire("LINCITY-SIMULATION-TRACE 1\nC 12\nC 12\nC 0\nP 1 12 12 " +
    std::to_string(GROUP_FIRE) + "\nP 2 13 12 " + std::to_string(GROUP_FIRE) +
    "\nP 3 12 13 " + std::to_string(GROUP_FIRE) + "\nP 4 13 13 " +
    std::to_string(GROUP_FIRE) + "\nZ\n");
  SimulationTrace trace(fire);
  world.do_random_fire();
  trace.finish();
  const auto started = dynamic_message_cast<FireStartedMessage>(world.popMessage());
  CHECK(started && started->getPoint() == site && started->getGroup().group == GROUP_BLACKSMITH);
  CHECK(started->str().find("(12,12)") != std::string::npos);
  CHECK(world.map(site)->getGroup() == GROUP_FIRE);
}

void vehicleTurns() {
  struct Turn { MapPoint entry, exit; int frame; bool left; };
  for(const auto& turn : std::array<Turn, 8>{{
    {{1, 0}, {0, 1}, 1, false}, {{0, 1}, {1, 0}, 1, true},
    {{1, 0}, {0, -1}, 3, true}, {{0, -1}, {1, 0}, 3, false},
    {{-1, 0}, {0, -1}, 5, false}, {{0, -1}, {-1, 0}, 5, true},
    {{-1, 0}, {0, 1}, 7, true}, {{0, 1}, {-1, 0}, 7, false}
  }}) {
    World world(32);
    const MapPoint start(site.x - turn.entry.x, site.y - turn.entry.y);
    const MapPoint end(site.x + turn.exit.x, site.y + turn.exit.y);
    const MapPoint beyond(end.x + turn.exit.x, end.y + turn.exit.y);
    for(const auto p : {start, site, end, beyond})
      trackConstructionGroup.placeItem(world, p);

    auto car = std::make_unique<Vehicle>(world, start, VEHICLE_BLUECAR);
    car->update(1);
    const auto began = car->anim + 1;
    car->update(began);
    CHECK(car->point == end && car->prev == site);
    const int expires = car->anim;
    CHECK(car->frameIt->frame == turn.frame && car->turn_left == turn.left);
    CHECK(expires - began == (turn.left ? 2250 : 750));

    car->update(began + (expires - began) / 4);
    const auto earlyX = car->xr, earlyY = car->yr;
    car->update(began + 3 * (expires - began) / 4);
    CHECK(car->point == end && car->next == beyond);
    CHECK(car->xr != earlyX || car->yr != earlyY);
    CHECK(car->frameIt->frame == turn.frame && world.map.is_visible(car->framePt));
  }
}

void transportMaintenance() {
  World world(32);
  const auto rail = site.e(3);
  roadConstructionGroup.placeItem(world, site);
  railConstructionGroup.placeItem(world, rail);
  auto& road = dynamic_cast<Transport&>(*world.map(site)->getConstruction());
  auto& tracks = *world.map(rail)->getConstruction();
  world.total_money = 100;
  world.total_time = 0;
  world.money_rates.transport_cost = 5;
  road.levelStuff(STUFF_GOODS, 3);
  road.levelStuff(STUFF_LOVOLT, 1);
  tracks.levelStuff(STUFF_GOODS, 3);
  tracks.levelStuff(STUFF_STEEL, 4);
  tracks.levelStuff(STUFF_LOVOLT, LOVOLT_LOSS_ON_TRANSPORT + 2);
  road.update();
  tracks.update();
  CHECK(world.total_money == 80 && world.stats.expenses.transport.acc == 20);
  CHECK(road.commodityCount[STUFF_GOODS] == 2 && road.commodityCount[STUFF_WASTE] == 1);
  CHECK(road.commodityCount[STUFF_LOVOLT] == 0 && world.map(site)->pollution == ROAD_POLLUTION);
  CHECK(tracks.commodityCount[STUFF_GOODS] == 2 && tracks.commodityCount[STUFF_STEEL] == 3);
  CHECK(tracks.commodityCount[STUFF_WASTE] == 2 && tracks.commodityCount[STUFF_LOVOLT] == 2);
  CHECK(world.map(rail)->pollution == RAIL_POLLUTION);

  world.total_money = -2000000000;
  world.total_time = 100;
  road.update();
  tracks.update();
  CHECK(world.total_money == -2000000000 && world.stats.expenses.transport.acc == 20);
  CHECK(road.canPlaceVehicle());
  {
    Vehicle parked(world, site, VEHICLE_BLUECAR);
    CHECK(!road.canPlaceVehicle());
  }
  CHECK(road.canPlaceVehicle());

  world.total_time = 1;
  const int threshold = roadConstructionGroup.commodityRuleCount[STUFF_WASTE].maxload * 9 / 10;
  const int pollution = world.map(site)->pollution;
  road.levelStuff(STUFF_WASTE, threshold);
  road.update();
  CHECK(road.commodityCount[STUFF_WASTE] == threshold);
  CHECK(world.map(site)->pollution == pollution);
  road.levelStuff(STUFF_WASTE, threshold + 1);
  std::istringstream input("LINCITY-SIMULATION-TRACE 1\nS transport.waste-fire 1\nP 1 12 12 " +
    std::to_string(GROUP_FIRE) + "\nZ\n");
  SimulationTrace trace(input);
  road.update();
  trace.finish();
  CHECK(world.map(site)->getGroup() == GROUP_FIRE);
  CHECK(world.map(site)->pollution == pollution + WASTE_BURN_ON_TRANSPORT_POLLUTE);
  const auto fire = dynamic_message_cast<FireStartedMessage>(world.popMessage());
  CHECK(fire && fire->getPoint() == site && fire->getGroup().group == GROUP_ROAD);
}

void inventionThreshold() {
  World world(32);
  const auto before = inventionsAfter(rocketPadConstructionGroup.tech - 1);
  CHECK(before.size() == 1 && before.front() == &rocketPadConstructionGroup);
  CHECK(inventionsAfter(rocketPadConstructionGroup.tech).empty());
  int prev = 0;
  for(const auto* group : inventionsAfter(0)) {
    CHECK(group->tech > 0 && group->tech >= prev);
    prev = group->tech;
  }
}

void sustainability() {
  World world(32);
  auto& stats = world.stats.sustainability;
  world.tech_level = SUST_MIN_TECH_LEVEL;
  stats.old_tech = world.tech_level;
  stats.old_population = SUST_MIN_POPULATION * NUMOF_DAYS_IN_MONTH;
  stats.mining_years = SUST_ORE_COAL_YEARS_NEEDED - 1;
  stats.trade_years = SUST_PORT_YEARS_NEEDED - 1;
  stats.money_years = SUST_MONEY_YEARS_NEEDED - 1;
  stats.population_years = SUST_POP_YEARS_NEEDED - 1;
  stats.tech_years = SUST_TECH_YEARS_NEEDED - 1;
  stats.fire_years = SUST_FIRE_YEARS_NEEDED - 3;
  world.total_time = 3 * NUMOF_DAYS_IN_YEAR - 1;
  blacksmithConstructionGroup.placeItem(world, site);
  world.map(site)->flags |= FLAG_FIRE_COVER;
  world.stats.population.population_m += stats.old_population;

  world.do_time_step();
  const auto gained = dynamic_message_cast<SustainableEconomyMessage>(world.popMessage());
  CHECK(gained && gained->getTime() == 3 * NUMOF_DAYS_IN_YEAR);
  CHECK(gained->str() == "the economy is sustainable");
  CHECK(stats.sustainable && stats.fire_years == SUST_FIRE_YEARS_NEEDED);
  CHECK(stats.population_years == SUST_POP_YEARS_NEEDED);

  world.total_time = 6 * NUMOF_DAYS_IN_YEAR - 1;
  world.map(site)->flags &= ~FLAG_FIRE_COVER;
  world.stats.population.population_m += stats.old_population;
  stats.mining_flag = stats.trade_flag = false;
  world.total_money = -1;
  world.tech_level = SUST_MIN_TECH_LEVEL - 1;
  world.do_time_step();
  CHECK(!world.popMessage());
  CHECK(stats.mining_flag && stats.trade_flag && stats.sustainable);
  CHECK(stats.mining_years == 0 && stats.trade_years == 0);
  CHECK(stats.money_years == 0 && stats.tech_years == 0 && stats.fire_years == 0);
}

void pollutionAndRecovery() {
  World world(32);
  auto& map = world.map;
  map(site)->pollution = 160;
  map(site.e())->pollution = 10;
  map(MapPoint(0, 1))->pollution = 64;
  world.scan_pollution();
  CHECK(world.stats.total_pollution == 234 && map.polluted.size() == 2);
  std::istringstream east("LINCITY-SIMULATION-TRACE 1\nC 8\nZ\n");
  SimulationTrace trace(east);
  world.do_pollution();
  trace.finish();
  CHECK(map(site)->pollution == 150 && map(site.e())->pollution == 20);
  CHECK(map(MapPoint(0, 1))->pollution == 1);

  map(site)->setTerrain(GROUP_DESERT);
  map(site)->flags |= FLAG_HAS_UNDERGROUND_WATER;
  std::istringstream regrow("LINCITY-SIMULATION-TRACE 1\nC 1\nZ\n");
  SimulationTrace recovery(regrow);
  world.do_daily_ecology();
  recovery.finish();
  CHECK(map(site)->group == GROUP_BARE);
}
}

int main() {
  try {
    terrainAndFootprints();
    mapSearchAndTerrain();
    riversAndCables();
    pollutionAndRecovery();
    sustainability();
    transportFrames();
    vehicleCrossings();
    vehicleTurns();
    transportMaintenance();
    inventionThreshold();
    eventDiagnostics();
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
