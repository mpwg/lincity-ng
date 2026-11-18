#include "lincity/modules/all_modules.hpp"
#include "lincity/sustainable.hpp"
#include "lincity/world.hpp"
#include "util/simulation_trace.hpp"

#include <array>
#include <iostream>
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
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
